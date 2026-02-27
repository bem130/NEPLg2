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
    /// open import が複数候補で曖昧。
    AmbiguousImport = 1101,
    /// 字句解析で未知のディレクティブ。
    LexerUnknownDirective = 1201,
    /// 字句解析で未知トークン。
    LexerUnknownToken = 1202,
    /// パーサでトークン期待に失敗。
    ParserExpectedToken = 2001,
    /// パーサで予期しないトークン。
    ParserUnexpectedToken = 2002,
    /// パーサで識別子期待に失敗。
    ParserExpectedIdentifier = 2003,
    /// パーサで型式の解釈に失敗。
    ParserInvalidTypeExpr = 2004,
    /// 予約語を識別子として使用。
    ParserReservedKeywordIdentifier = 2005,
    /// #extern シグネチャが不正。
    ParserInvalidExternSignature = 2006,
    /// 未定義識別子。
    TypeUndefinedIdentifier = 3001,
    /// 未定義変数。
    TypeUndefinedVariable = 3002,
    /// 返り値型がシグネチャと不一致。
    TypeReturnTypeMismatch = 3003,
    /// 型注釈不一致。
    TypeAnnotationMismatch = 3004,
    /// オーバーロード解決が曖昧。
    TypeAmbiguousOverload = 3005,
    /// オーバーロード候補に一致なし。
    TypeNoMatchingOverload = 3006,
    /// match 対象が enum ではない。
    TypeMatchScrutineeMustBeEnum = 3007,
    /// match の arm が重複。
    TypeDuplicateMatchArm = 3008,
    /// match が非網羅。
    TypeNonExhaustiveMatch = 3009,
    /// 不変変数への代入。
    TypeImmutableMutation = 3010,
    /// 不正なフィールドアクセス。
    TypeInvalidFieldAccess = 3011,
    /// 不明 intrinsic。
    TypeUnknownIntrinsic = 3012,
    /// pipe 使用エラー。
    TypePipeError = 3013,
    /// non-shadowable への shadow 試行。
    TypeNoShadowViolation = 3014,
    /// no-shadow 宣言の衝突。
    TypeNoShadowConflict = 3015,
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
            1101 => Some(DiagnosticId::AmbiguousImport),
            1201 => Some(DiagnosticId::LexerUnknownDirective),
            1202 => Some(DiagnosticId::LexerUnknownToken),
            2001 => Some(DiagnosticId::ParserExpectedToken),
            2002 => Some(DiagnosticId::ParserUnexpectedToken),
            2003 => Some(DiagnosticId::ParserExpectedIdentifier),
            2004 => Some(DiagnosticId::ParserInvalidTypeExpr),
            2005 => Some(DiagnosticId::ParserReservedKeywordIdentifier),
            2006 => Some(DiagnosticId::ParserInvalidExternSignature),
            3001 => Some(DiagnosticId::TypeUndefinedIdentifier),
            3002 => Some(DiagnosticId::TypeUndefinedVariable),
            3003 => Some(DiagnosticId::TypeReturnTypeMismatch),
            3004 => Some(DiagnosticId::TypeAnnotationMismatch),
            3005 => Some(DiagnosticId::TypeAmbiguousOverload),
            3006 => Some(DiagnosticId::TypeNoMatchingOverload),
            3007 => Some(DiagnosticId::TypeMatchScrutineeMustBeEnum),
            3008 => Some(DiagnosticId::TypeDuplicateMatchArm),
            3009 => Some(DiagnosticId::TypeNonExhaustiveMatch),
            3010 => Some(DiagnosticId::TypeImmutableMutation),
            3011 => Some(DiagnosticId::TypeInvalidFieldAccess),
            3012 => Some(DiagnosticId::TypeUnknownIntrinsic),
            3013 => Some(DiagnosticId::TypePipeError),
            3014 => Some(DiagnosticId::TypeNoShadowViolation),
            3015 => Some(DiagnosticId::TypeNoShadowConflict),
            _ => None,
        }
    }

    /// 診断IDに対応する短い説明を返します。
    pub const fn message(self) -> &'static str {
        match self {
            DiagnosticId::MultipleTargetDirective => "multiple #target directives are not allowed",
            DiagnosticId::UnknownTargetDirective => "unknown target in #target",
            DiagnosticId::LoaderFailure => "loader error",
            DiagnosticId::AmbiguousImport => "ambiguous import",
            DiagnosticId::LexerUnknownDirective => "unknown directive",
            DiagnosticId::LexerUnknownToken => "unknown token",
            DiagnosticId::ParserExpectedToken => "expected token",
            DiagnosticId::ParserUnexpectedToken => "unexpected token",
            DiagnosticId::ParserExpectedIdentifier => "expected identifier",
            DiagnosticId::ParserInvalidTypeExpr => "invalid type expression",
            DiagnosticId::ParserReservedKeywordIdentifier => {
                "reserved keyword cannot be used as identifier"
            }
            DiagnosticId::ParserInvalidExternSignature => "invalid #extern signature",
            DiagnosticId::TypeUndefinedIdentifier => "undefined identifier",
            DiagnosticId::TypeUndefinedVariable => "undefined variable",
            DiagnosticId::TypeReturnTypeMismatch => "return type does not match signature",
            DiagnosticId::TypeAnnotationMismatch => "type annotation mismatch",
            DiagnosticId::TypeAmbiguousOverload => "ambiguous overload",
            DiagnosticId::TypeNoMatchingOverload => "function signature does not match any overload",
            DiagnosticId::TypeMatchScrutineeMustBeEnum => "match scrutinee must be an enum",
            DiagnosticId::TypeDuplicateMatchArm => "duplicate match arm",
            DiagnosticId::TypeNonExhaustiveMatch => "non-exhaustive match",
            DiagnosticId::TypeImmutableMutation => "immutable mutation",
            DiagnosticId::TypeInvalidFieldAccess => "cannot access field on this type",
            DiagnosticId::TypeUnknownIntrinsic => "unknown intrinsic",
            DiagnosticId::TypePipeError => "pipe usage error",
            DiagnosticId::TypeNoShadowViolation => "cannot shadow non-shadowable symbol",
            DiagnosticId::TypeNoShadowConflict => "noshadow declaration conflicts",
        }
    }

}

/// 既存呼び出し互換: 数値IDから短い説明を返します。
pub fn message(id: u32) -> Option<&'static str> {
    DiagnosticId::from_u32(id).map(|d| d.message())
}
