//! Output `MBTiles` creation and writing.
//!
//! Provides helpers for creating a new `.mbtiles` (`SQLite`) file with the standard
//! schema and inserting tile blobs.

use std::path::Path;

use anyhow::Context;
use rusqlite::Connection;

/// Create a new `MBTiles` file with the standard schema.
///
/// Creates `metadata` and `tiles` tables with appropriate indices.
/// Overwrites any existing file at `path`.
pub fn create_mbtiles(path: &Path) -> anyhow::Result<Connection> {
    // Remove existing file if present.
    if path.exists() {
        std::fs::remove_file(path)
            .with_context(|| format!("remove existing MBTiles {}", path.display()))?;
    }

    let conn =
        Connection::open(path).with_context(|| format!("create MBTiles {}", path.display()))?;

    conn.execute_batch(
        "CREATE TABLE metadata (name TEXT, value TEXT);
         CREATE UNIQUE INDEX metadata_idx ON metadata (name);
         CREATE TABLE tiles (
             zoom_level INTEGER,
             tile_column INTEGER,
             tile_row INTEGER,
             tile_data BLOB
         );
         CREATE UNIQUE INDEX tile_index ON tiles (zoom_level, tile_column, tile_row);",
    )
    .context("create MBTiles schema")?;

    Ok(conn)
}

/// Insert a single tile blob into an open `MBTiles` connection.
pub fn insert_tile(
    conn: &Connection,
    zoom: i32,
    col: i32,
    row: i32,
    data: &[u8],
) -> anyhow::Result<()> {
    conn.execute(
        "INSERT INTO tiles (zoom_level, tile_column, tile_row, tile_data) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![zoom, col, row, data],
    )
    .context("insert tile")?;
    Ok(())
}

/// Set a metadata key-value pair.
pub fn set_metadata(conn: &Connection, name: &str, value: &str) -> anyhow::Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO metadata (name, value) VALUES (?1, ?2)",
        rusqlite::params![name, value],
    )
    .context("set metadata")?;
    Ok(())
}

/// Copy all metadata rows from `src` to `dst`.
pub fn copy_metadata(src: &Connection, dst: &Connection) -> anyhow::Result<()> {
    let mut stmt = src.prepare("SELECT name, value FROM metadata")?;
    let rows = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        let value: String = row.get(1)?;
        Ok((name, value))
    })?;

    for row in rows {
        let (name, value) = row?;
        set_metadata(dst, &name, &value)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_insert_read() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.mbtiles");

        let conn = create_mbtiles(&path).unwrap();
        set_metadata(&conn, "name", "test").unwrap();
        insert_tile(&conn, 5, 10, 20, b"hello").unwrap();

        // Read back.
        let data: Vec<u8> = conn
            .query_row(
                "SELECT tile_data FROM tiles WHERE zoom_level=5 AND tile_column=10 AND tile_row=20",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(data, b"hello");

        let name: String = conn
            .query_row("SELECT value FROM metadata WHERE name='name'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(name, "test");
    }

    #[test]
    fn copy_metadata_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let src_path = dir.path().join("src.mbtiles");
        let dst_path = dir.path().join("dst.mbtiles");

        let src = create_mbtiles(&src_path).unwrap();
        set_metadata(&src, "name", "source").unwrap();
        set_metadata(&src, "format", "pbf").unwrap();

        let dst = create_mbtiles(&dst_path).unwrap();
        copy_metadata(&src, &dst).unwrap();
        set_metadata(&dst, "format", "mlt").unwrap();

        let format: String = dst
            .query_row(
                "SELECT value FROM metadata WHERE name='format'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(format, "mlt");

        let name: String = dst
            .query_row("SELECT value FROM metadata WHERE name='name'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(name, "source");
    }
}
