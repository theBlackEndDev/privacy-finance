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
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::recurring::get_all_recurring,
            commands::recurring::get_recurring,
            commands::recurring::create_recurring,
            commands::recurring::update_recurring,
            commands::recurring::delete_recurring,
            commands::recurring::toggle_recurring_active,
            commands::recurring::check_due_recurring,
            commands::recurring::generate_recurring_transactions,
            commands::search::search_transactions,
            commands::search::search_transactions_full,
            commands::search::search_suggest,
            commands::search::rebuild_fts,
            commands::export::export_transactions_csv,
            commands::export::export_transactions_json,
            commands::export::export_all_data_json,
            commands::backup::create_backup,
            commands::backup::restore_backup,
            commands::backup::get_backup_info,
            commands::budget::create_budget,
            commands::budget::get_budget,
            commands::budget::update_budget_item,
            commands::budget::copy_budget,
            commands::budget::get_budget_summary,
            commands::budget::calculate_rollover,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
