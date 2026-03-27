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
            commands::categories::list_category_groups,
            commands::categories::create_category_group,
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
            commands::settings::delete_all_data,
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
            commands::budget::get_category_activity,
            commands::rules::create_rule,
            commands::rules::get_all_rules,
            commands::rules::update_rule,
            commands::rules::delete_rule,
            commands::rules::reorder_rules,
            commands::rules::match_transaction,
            commands::rules::apply_rules_to_uncategorized,
            commands::rules::test_rule,
            commands::splits::create_splits,
            commands::splits::get_splits,
            commands::splits::update_splits,
            commands::splits::delete_splits,
            commands::tags::create_tag,
            commands::tags::get_all_tags,
            commands::tags::update_tag,
            commands::tags::delete_tag,
            commands::tags::tag_transaction,
            commands::tags::untag_transaction,
            commands::tags::get_transaction_tags,
            commands::tags::get_transactions_by_tag,
            commands::reports::get_income_expense_report,
            commands::reports::get_category_spending_report,
            commands::reports::get_cash_flow_report,
            commands::reports::get_net_worth_report,
            commands::reports::get_tag_spending_report,
            commands::reports::get_payee_spending_report,
            commands::reconciliation::start_reconciliation,
            commands::reconciliation::toggle_reconciliation_item,
            commands::reconciliation::get_reconciliation_diff,
            commands::reconciliation::complete_reconciliation,
            commands::reconciliation::cancel_reconciliation,
            commands::exchange_rates::create_rate,
            commands::exchange_rates::get_rate,
            commands::exchange_rates::get_all_rates,
            commands::exchange_rates::delete_rate,
            commands::exchange_rates::convert_amount,
            commands::encryption::check_encryption_status,
            commands::encryption::enable_encryption,
            commands::encryption::unlock_database,
            commands::encryption::change_password,
            commands::encryption::disable_encryption,
            // AI Search (natural language)
            commands::ai_search::ai_search,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
