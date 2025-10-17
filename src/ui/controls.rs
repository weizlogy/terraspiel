use egui::Context;

pub fn draw_ui(ctx: &Context) -> bool {
    let mut should_quit = false;
    egui::Window::new("Hello").show(ctx, |ui| {
        ui.label("This is a minimal sample for Terraspiel.");
        ui.separator();
        if ui.button("Quit").clicked() {
            should_quit = true;
        }
    });
    should_quit
}
