module.exports = grammar({
  name: "neplg2",

  extras: $ => [
    /[ \t\r]+/,
  ],

  rules: {
    source_file: $ => repeat($._item),

    _item: $ => choice(
      $.comment,
      $.doc_comment,
      $.directive,
      $.function_definition,
      $.block,
      $.expression,
      $.newline,
    ),

    newline: _ => /\n+/,

    comment: _ => token(seq("//", /.*/)),
    doc_comment: _ => token(seq("//:", /.*/)),
    directive: _ => token(seq("#", /[^\n]+/)),

    function_definition: $ => seq(
      "fn",
      field("name", $.identifier),
      repeat(choice($.type_annotation, $.parameter_list)),
      ":",
      field("body", $.block)
    ),

    parameter_list: $ => seq(
      "(",
      optional(seq($.identifier, repeat(seq(",", $.identifier)))),
      ")"
    ),

    type_annotation: $ => seq(
      "<",
      repeat1(choice($.type_identifier, $.identifier, $.operator, $.number, $.string)),
      ">"
    ),

    block: $ => repeat1(choice($.expression, $.comment, $.doc_comment, $.newline)),
    match_arm_block: $ => $.block,

    expression: $ => repeat1(choice(
      $.keyword,
      $.identifier,
      $.type_identifier,
      $.number,
      $.string,
      $.operator,
      $.type_annotation,
      $.parameter_list
    )),

    keyword: _ => choice(
      "fn",
      "let",
      "mut",
      "set",
      "if",
      "then",
      "else",
      "while",
      "match",
      "struct",
      "enum",
      "trait",
      "impl",
      "block",
      "do",
      "cond"
    ),

    identifier: _ => /[A-Za-z_][A-Za-z0-9_]*/,
    type_identifier: _ => /[A-Z][A-Za-z0-9_]*/,
    number: _ => /-?[0-9]+(\.[0-9]+)?/,
    string: _ => /"([^"\\]|\\.)*"/,
    operator: _ => choice("->", "*>", "::", ";", ":", ",", ".", "@", "&", "*", "-", "="),
  }
});
