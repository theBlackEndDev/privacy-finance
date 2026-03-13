mod db;
mod error;
mod helpers;
mod commands;
mod models;

use db::init_db;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let state = init_db(app.handle())
                .expect("failed to initialize database");
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::accounts::get_accounts,
            commands::accounts::get_account,
            commands::accounts::create_account,
            commands::accounts::update_account,
            commands::accounts::delete_account,
            commands::categories::get_categories,
            commands::categories::create_category,
            commands::categories::update_category,
            commands::categories::delete_category,
            commands::transactions::create_transaction,
            commands::transactions::get_transaction,
            commands::transactions::update_transaction,
            commands::transactions::delete_transaction,
            commands::transactions::get_transactions,
            commands::transactions::get_payees,
            commands::transactions::create_transfer,
            commands::dashboard::get_dashboard_data,
            commands::dashboard::get_monthly_trend,
            commands::dashboard::get_account_balances,
            commands::import::preview_csv,
            commands::import::import_csv,
            commands::import::rollback_import,
            commands::import::get_import_batches,
            commands::import::get_import_profiles,
            commands::import::save_import_profile,
            commands::import::delete_import_profile,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
