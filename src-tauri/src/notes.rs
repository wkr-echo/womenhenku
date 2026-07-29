// Notes service layer — Stage 4
//
// Provides thin wrappers around NoteRepository for use by Tauri commands.
// All database access goes through the repository layer.

use crate::db::model::Note;
use crate::db::repository::NoteRepository;
use crate::db::DbPool;

/// Save or update a note for an entry (upsert by entry_id).
pub fn save_note(pool: &DbPool, entry_id: i64, content: &str) -> Result<Note, String> {
    let repo = NoteRepository::new(pool.clone());
    repo.save(entry_id, content).map_err(|e| e.to_string())
}

/// Get a note by its entry_id. Returns None if no note exists.
pub fn get_note(pool: &DbPool, entry_id: i64) -> Result<Option<Note>, String> {
    let repo = NoteRepository::new(pool.clone());
    repo.find_by_entry_id(entry_id).map_err(|e| e.to_string())
}

/// Delete a note by its entry_id.
pub fn delete_note(pool: &DbPool, entry_id: i64) -> Result<(), String> {
    let repo = NoteRepository::new(pool.clone());
    repo.delete(entry_id).map_err(|e| e.to_string())
}