//! 数据库连接与路径管理。

use std::env;
use std::path::PathBuf;

use rusqlite::Connection;

use crate::schema;

/// 解析数据库文件路径。
///
/// 优先级：`RECALL_DB` 环境变量 > 平台默认路径。
/// macOS 默认 `~/Library/Application Support/learnsys/learnsys.db`。
/// Docker 部署时挂卷覆盖此路径即可。
pub fn db_path() -> PathBuf {
    if let Ok(p) = env::var("RECALL_DB") {
        return PathBuf::from(p);
    }
    let home = env::var("HOME").expect("HOME 环境变量未设置");
    PathBuf::from(home).join("Library/Application Support/learnsys/learnsys.db")
}

/// 打开（必要时创建父目录）并初始化数据库。
pub fn connect() -> Result<Connection, rusqlite::Error> {
    let path = db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let conn = Connection::open(&path)?;
    schema::init(&conn)?;
    Ok(conn)
}
