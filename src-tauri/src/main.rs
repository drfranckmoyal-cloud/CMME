// Pas de fenêtre console supplémentaire en version publiée.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    cmme_app_lib::run()
}
