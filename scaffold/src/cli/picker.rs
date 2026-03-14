// WARNING: Generated code could not be parsed by syn for formatting.
// Run `cargo fmt` manually after fixing any syntax issues.



// TODO: package-level var — consider lazy_static! or OnceLock
// Go: var titleStyle = lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("12"))
fn title_style_init() -> Style /* todo: lipgloss.Style */ {
    todo!("package-level var init")
}
// TODO: package-level var — consider lazy_static! or OnceLock
// Go: var itemStyle = lipgloss.NewStyle().PaddingLeft(2)
fn item_style_init() -> Style /* todo: lipgloss.Style */ {
    todo!("package-level var init")
}
// TODO: package-level var — consider lazy_static! or OnceLock
// Go: var selectedStyle = lipgloss.NewStyle().PaddingLeft(0).Foreground(lipgloss.Color("10")).Bold(true)
fn selected_style_init() -> Style /* todo: lipgloss.Style */ {
    todo!("package-level var init")
}
// TODO: package-level var — consider lazy_static! or OnceLock
// Go: var cursorStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("10"))
fn cursor_style_init() -> Style /* todo: lipgloss.Style */ {
    todo!("package-level var init")
}
// TODO: package-level var — consider lazy_static! or OnceLock
// Go: var subtleStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("8"))
fn subtle_style_init() -> Style /* todo: lipgloss.Style */ {
    todo!("package-level var init")
}
#[derive(Debug, Clone, Default)]
struct pickerModel {
    pub(crate) title: String,
    pub(crate) items: Vec<String>,
    pub(crate) cursor: i64,
    pub(crate) choice: String,
    pub(crate) quitting: bool,
}

fn new_picker_model(title: &str, items: &[String]) -> pickerModel /* todo: cli.pickerModel */ {
    pickerModel /* todo: cli.pickerModel */ { title: title, items: items, ..Default::default() }
}

/// RunPicker launches an interactive Bubble Tea list picker and returns the
/// selected item, or an empty string if the user cancelled.
pub fn run_picker(title: &str, items: &[String]) -> anyhow::Result<String> {
    if items.len() == 0 {
        return Err((anyhow::anyhow!("no items to pick from")).into());
    }
    let mut m = new_picker_model(title, items);
    let mut p = tea.new_program(m);
    let mut result = p.run()?;
    // if err != nil { ... } — handled by ? above
    let mut final = /* type assert */ result.downcast_ref::<pickerModel /* todo: cli.pickerModel */>();
    Ok(final.choice)
}


impl pickerModel {
    pub fn init(&self) -> Cmd /* todo: bubbletea.Cmd */ {
        None
    }

    pub fn update(&self, msg: Msg /* todo: bubbletea.Msg */) -> (Model /* todo: bubbletea.Model */, Cmd /* todo: bubbletea.Cmd */) {
        // type switch — requires manual translation
        let mut msg = msg /* type switch */;
        // case tea.key_msg
        {
            match msg.string() {
                "ctrl+c" | "q" | "esc" => {
                    self.quitting = true;
                    (self, tea.quit)
                }
                "up" | "k" => {
                    if self.cursor > 0 {
                        self.cursor -= 1;
                    }
                }
                "down" | "j" => {
                    if self.cursor < self.items.len() - 1 {
                        self.cursor += 1;
                    }
                }
                "enter" => {
                    self.choice = self.items[self.cursor];
                    (self, tea.quit)
                }
            }
        }
        (self, None)
    }

    pub fn view(&self) -> String {
        if self.quitting && self.choice == "" {
            subtle_style.render("Cancelled.") + "\n"
        }
        if self.choice != "" {
            ""
        }
        let mut s = title_style.render(self.title) + "\n\n";
        for (i, item) in self.items.iter().enumerate() {
            if i == self.cursor {
                s += cursor_style.render("▸ ") + selected_style.render(item) + "\n";
            } else {
                s += item_style.render("  " + item) + "\n";
            }
        }
        s += "\n" + subtle_style.render("↑/↓ navigate • enter select • q quit") + "\n";
        s
    }

}
