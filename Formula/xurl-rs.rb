class XurlRs < Formula
  desc "Fast, ergonomic CLI for the X (Twitter) API — the Rust port of xurl"
  homepage "https://github.com/brettdavies/xurl-rs"
  url "https://github.com/brettdavies/xurl-rs/archive/refs/tags/v1.0.3.tar.gz"
  license "MIT"
  head "https://github.com/brettdavies/xurl-rs.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
    # Install shell completions
    generate_completions_from_executable(bin/"xr", "--generate-completion")
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/xr --version")
  end
end
