//! A spawned program and every process it starts, ended as one.
//!
//! Killing the program alone leaves whatever it started running with no
//! parent. On Unix the program leads a process group of its own, and the
//! group is killed with `killpg`. On Windows it starts suspended, joins a
//! Job Object that kills every process in it when its last handle closes,
//! and only then runs.

use std::io;

use tokio::process::{Child, Command};

/// A running program and the processes it starts. Dropping it before
/// [`ProcessTree::release`] kills them all: a timed-out attempt, the drain
/// deadline, and a second signal each end in that drop.
pub(super) struct ProcessTree {
    pub(super) child: Child,
    armed: bool,
    #[cfg(windows)]
    job: windows::Job,
}

impl ProcessTree {
    /// Spawns `command` in a tree of its own.
    pub(super) fn spawn(mut command: Command) -> io::Result<Self> {
        command.kill_on_drop(true);
        #[cfg(unix)]
        {
            command.process_group(0);
            let child = command.spawn()?;
            Ok(Self { child, armed: true })
        }
        #[cfg(windows)]
        {
            command.creation_flags(windows::CREATE_SUSPENDED);
            let mut child = command.spawn()?;
            match windows::Job::adopt(&child) {
                Ok(job) => Ok(Self {
                    child,
                    armed: true,
                    job,
                }),
                Err(error) => {
                    let _ = child.start_kill();
                    Err(error)
                }
            }
        }
    }

    /// The program has exited: what it started is left alone.
    pub(super) fn release(&mut self) {
        self.armed = false;
        #[cfg(windows)]
        self.job.release();
    }
}

impl Drop for ProcessTree {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        #[cfg(unix)]
        if let Some(Ok(group)) = self.child.id().map(libc::pid_t::try_from) {
            // SAFETY: killpg takes no pointers; the group is the one the
            // program leads, made at spawn, and it has not been reaped.
            unsafe {
                libc::killpg(group, libc::SIGKILL);
            }
        }
        #[cfg(windows)]
        self.job.terminate();
    }
}

#[cfg(windows)]
mod windows {
    use std::io;
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};

    use tokio::process::Child;
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
    };
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
        SetInformationJobObject, TerminateJobObject,
    };
    use windows_sys::Win32::System::Threading::{OpenThread, ResumeThread, THREAD_SUSPEND_RESUME};

    pub(super) use windows_sys::Win32::System::Threading::CREATE_SUSPENDED;

    /// A kill-on-close Job Object holding one program.
    pub(super) struct Job(OwnedHandle);

    impl Job {
        /// Puts the suspended `child` in a new job, then lets it run, so
        /// nothing it starts can escape the job.
        pub(super) fn adopt(child: &Child) -> io::Result<Self> {
            // SAFETY: both arguments may be null: no security attributes,
            // no name.
            let raw = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
            if raw.is_null() {
                return Err(io::Error::last_os_error());
            }
            // SAFETY: `raw` is a job handle this function owns from here on.
            let job = Self(unsafe { OwnedHandle::from_raw_handle(raw) });
            job.limit(JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE)?;
            let process = child
                .raw_handle()
                .ok_or_else(|| io::Error::other("the program exited before it ran"))?;
            // SAFETY: both handles are open: the job's is owned above and the
            // process's belongs to `child`, which outlives this call.
            if unsafe { AssignProcessToJobObject(job.0.as_raw_handle(), process) } == 0 {
                return Err(io::Error::last_os_error());
            }
            let pid = child
                .id()
                .ok_or_else(|| io::Error::other("the program exited before it ran"))?;
            resume(pid)?;
            Ok(job)
        }

        /// Ends every process in the job.
        pub(super) fn terminate(&self) {
            // SAFETY: the handle is open for as long as `self` lives.
            unsafe {
                TerminateJobObject(self.0.as_raw_handle(), 1);
            }
        }

        /// Clears kill-on-close, so closing the handle ends nothing.
        pub(super) fn release(&self) {
            let _ = self.limit(0);
        }

        fn limit(&self, flags: u32) -> io::Result<()> {
            let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            limits.BasicLimitInformation.LimitFlags = flags;
            let size = u32::try_from(std::mem::size_of_val(&limits))
                .map_err(|_| io::Error::other("job limits too large"))?;
            // SAFETY: the pointer and size describe `limits`, which lives
            // across the call, and the handle is open.
            let set = unsafe {
                SetInformationJobObject(
                    self.0.as_raw_handle(),
                    JobObjectExtendedLimitInformation,
                    std::ptr::from_ref(&limits).cast(),
                    size,
                )
            };
            if set == 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        }
    }

    /// Resumes every thread of the suspended process `pid`: the one it was
    /// created with.
    fn resume(pid: u32) -> io::Result<()> {
        // SAFETY: a thread snapshot takes no pointers; 0 means all processes.
        let raw = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
        if raw == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: `raw` is a snapshot handle this function owns from here on.
        let snapshot = unsafe { OwnedHandle::from_raw_handle(raw) };
        let mut entry = THREADENTRY32 {
            dwSize: u32::try_from(std::mem::size_of::<THREADENTRY32>())
                .map_err(|_| io::Error::other("thread entry too large"))?,
            ..THREADENTRY32::default()
        };
        let mut resumed = false;
        // SAFETY: `entry` is a THREADENTRY32 with its size set, as both calls
        // require, and the snapshot handle is open.
        let mut more = unsafe { Thread32First(snapshot.as_raw_handle(), &mut entry) } != 0;
        while more {
            if entry.th32OwnerProcessID == pid {
                // SAFETY: OpenThread takes no pointers; a null result is
                // checked before use, and the handle is closed after.
                unsafe {
                    let thread = OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID);
                    if !thread.is_null() {
                        resumed |= ResumeThread(thread) != u32::MAX;
                        CloseHandle(thread);
                    }
                }
            }
            // SAFETY: as for Thread32First.
            more = unsafe { Thread32Next(snapshot.as_raw_handle(), &mut entry) } != 0;
        }
        if resumed {
            Ok(())
        } else {
            Err(io::Error::other(
                "the suspended program's thread was not found",
            ))
        }
    }
}
