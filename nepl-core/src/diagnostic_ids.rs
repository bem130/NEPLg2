//! 診断IDの定義テーブル。
//!
//! 数値IDは表示層（CLI/LSP/Web）で共通利用し、短いIDから詳細説明へ
//! 参照できるようにするための土台です。

/// 診断ID。
///
/// 表示時は `D{number}` 形式（例: `D1001`）で扱います。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DiagnosticId {
    /// #target が複数回指定された。
    MultipleTargetDirective = 1001,
    /// #target の値が不正。
    UnknownTargetDirective = 1002,
    /// VFS/Loader でソース取得に失敗。
    LoaderFailure = 1003,
}

impl DiagnosticId {
    /// 数値IDへ変換します。
    pub const fn as_u32(self) -> u32 {
        self as u32
    }

    /// 数値IDから列挙値へ変換します。
    pub const fn from_u32(id: u32) -> Option<DiagnosticId> {
        match id {
            1001 => Some(DiagnosticId::MultipleTargetDirective),
            1002 => Some(DiagnosticId::UnknownTargetDirective),
            1003 => Some(DiagnosticId::LoaderFailure),
            _ => None,
        }
    }

    /// 診断IDに対応する短い説明を返します。
    pub const fn message(self) -> &'static str {
        match self {
            DiagnosticId::MultipleTargetDirective => "multiple #target directives are not allowed",
            DiagnosticId::UnknownTargetDirective => "unknown target in #target",
            DiagnosticId::LoaderFailure => "loader error",
        }
    }
}

/// 既存呼び出し互換: 数値IDから短い説明を返します。
pub fn message(id: u32) -> Option<&'static str> {
    DiagnosticId::from_u32(id).map(|d| d.message())
}
