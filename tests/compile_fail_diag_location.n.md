# compile_fail diagnostic location

## compile_fail_matches_diag_id_and_line_col

neplg2:test[compile_fail]
diag_id: 3001
diag_span: 3:5
```neplg2
#entry main
fn main <()->i32> ():
    missing_name
```

## compile_fail_matches_multiple_diag_spans

neplg2:test[compile_fail]
diag_id: 3001
diag_spans: ["3:5", "/virtual/entry.nepl:3:5"]
```neplg2
#entry main
fn main <()->i32> ():
    missing_name
```

## compile_fail_matches_object_style_diag_spans

neplg2:test[compile_fail]
diag_id: 3001
diag_spans: [{"line": 3, "col": 5}, {"file": "/virtual/entry.nepl", "line": 3, "col": 5}]
```neplg2
#entry main
fn main <()->i32> ():
    missing_name
```
