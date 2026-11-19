mod db;
mod error;
mod helpers;
mod commands;
mod models;

use db::init_db;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init());

    #[cfg(feature = "e2e")]
    {
        eprintln!("[E2E] Registering tauri-plugin-webdriver on port 4445");
        builder = builder.plugin(tauri_plugin_webdriver::init());
    }

    builder
        .setup(|app| {
            let state = init_db(app.handle())
                .expect("failed to initialize database");

            // Initialize inference server state for Tier 2 AI search
            let app_data_dir = app.handle().path().app_data_dir()
                .expect("failed to get app data dir");
            app.manage(commands::ai_model::InferenceServerState {
                app_data_dir,
            });

            // Lazily loaded on the first Tier 2 query
            app.manage(std::sync::Mutex::new(None::<helpers::ai_engine::Engine>));

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
            commands::import::get_import_profiles,
            commands::import::save_import_profile,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::delete_all_data,
            commands::recurring::get_all_recurring,
            commands::recurring::create_recurring,
            commands::recurring::update_recurring,
            commands::recurring::delete_recurring,
            commands::recurring::toggle_recurring_active,
            commands::recurring::check_due_recurring,
            commands::recurring::generate_recurring_transactions,
            commands::search::search_transactions_full,
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
            commands::reports::get_income_expense_report,
            commands::reports::get_category_spending_report,
            commands::reports::get_cash_flow_report,
            commands::reports::get_net_worth_report,
            commands::reports::get_tag_spending_report,
            commands::reports::get_payee_spending_report,
            commands::reconciliation::start_reconciliation,
            commands::reconciliation::toggle_reconciliation_item,
            commands::reconciliation::complete_reconciliation,
            commands::reconciliation::cancel_reconciliation,
            commands::exchange_rates::create_rate,
            commands::exchange_rates::get_all_rates,
            commands::exchange_rates::delete_rate,
            commands::encryption::check_encryption_status,
            commands::encryption::enable_encryption,
            commands::encryption::unlock_database,
            commands::encryption::change_password,
            commands::encryption::disable_encryption,
            // AI Search (natural language)
            commands::ai_search::ai_search,
            commands::ai_model::ai_model_status,
            commands::ai_model::ai_model_download,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
