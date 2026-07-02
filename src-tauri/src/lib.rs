pub mod bases;
pub mod commands;
pub mod db;
pub mod error;
pub mod folders;
pub mod link_parser;
pub mod links;
pub mod models;
pub mod notes;
pub mod slug;
pub mod state;
pub mod tags;
pub mod vault_registry;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::vaults::list_vaults,
            commands::vaults::create_vault,
            commands::vaults::open_vault,
            commands::vaults::close_vault,
            commands::vaults::current_vault,
            commands::vaults::delete_vault,
            commands::folders::list_folders,
            commands::folders::create_folder,
            commands::folders::rename_folder,
            commands::folders::move_folder,
            commands::folders::delete_folder,
            commands::notes::create_note,
            commands::notes::get_note,
            commands::notes::update_note_content,
            commands::notes::rename_note,
            commands::notes::move_note,
            commands::notes::set_note_pinned,
            commands::notes::set_note_archived,
            commands::notes::set_note_properties,
            commands::notes::set_note_color,
            commands::notes::delete_note,
            commands::notes::list_notes,
            commands::notes::list_note_versions,
            commands::notes::restore_note_version,
            commands::notes::create_snapshot,
            commands::notes::search_notes,
            commands::notes::get_backlinks,
            commands::notes::get_outgoing_links,
            commands::notes::get_all_links,
            commands::tags::list_tags,
            commands::tags::list_tags_for_note,
            commands::tags::list_notes_for_tag,
            commands::tags::list_all_note_tags,
            commands::tags::add_tag_to_note,
            commands::tags::remove_tag_from_note,
            commands::tags::set_tag_color,
            commands::bases::list_bases,
            commands::bases::create_base,
            commands::bases::rename_base,
            commands::bases::set_base_color,
            commands::bases::delete_base,
            commands::bases::add_note_to_base,
            commands::bases::remove_note_from_base,
            commands::bases::list_all_base_notes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
