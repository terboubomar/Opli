mod recipe;

#[tauri::command]
fn preview_recipe(
    paths: Vec<String>,
    recipe_id: String,
    output_directory: String,
    store: tauri::State<'_, recipe::PlanStore>,
) -> Result<recipe::Preview, String> {
    store.preview(&paths, &recipe_id, &output_directory)
}

#[tauri::command]
fn execute_recipe(
    plan_id: String,
    store: tauri::State<'_, recipe::PlanStore>,
) -> Result<recipe::RunResult, String> {
    store.execute(&plan_id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(recipe::PlanStore::default())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![preview_recipe, execute_recipe])
        .run(tauri::generate_context!())
        .expect("error while starting Intent Layer");
}
