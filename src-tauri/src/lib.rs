//! The Mealprep application: Tauri commands over the business rules, stored on the device.

mod command;
mod error;
mod result;
mod setup;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().plugin(tauri_plugin_opener::init());
    #[cfg(mobile)]
    let builder = builder.plugin(tauri_plugin_barcode_scanner::init());

    builder
        .setup(|app| {
            let state = tauri::async_runtime::block_on(setup::open(app.handle()))?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            command::calendar::assign_calendar_day,
            command::calendar::list_calendar_days,
            command::calendar::unassign_calendar_day,
            command::category::create_category,
            command::category::delete_category,
            command::category::list_categories,
            command::category::rename_category,
            command::day_plan::create_day_plan,
            command::day_plan::delete_day_plan,
            command::day_plan::get_day_plan,
            command::day_plan::list_day_plans,
            command::day_plan::update_day_plan,
            command::meal::create_meal,
            command::meal::delete_meal,
            command::meal::get_meal,
            command::meal::list_meals,
            command::meal::update_meal,
            command::product::create_product,
            command::product::delete_product,
            command::product::get_catalogue_entry,
            command::product::get_product,
            command::product::import_product,
            command::product::list_products,
            command::product::search_catalogue,
            command::product::update_product,
            command::targets::get_targets,
            command::targets::set_targets,
            command::weight::delete_weight_entry,
            command::weight::list_weight_entries,
            command::weight::set_weight_entry,
            command::weight::weight_series,
        ])
        .run(tauri::generate_context!())
        .expect("internal error: cannot run the Tauri application");
}
