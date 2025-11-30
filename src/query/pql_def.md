
query -> group | expr
group -> "(" query ")"
expr -> stmt [ expr_op query ]
expr_op -> "&" | "|"
stmt -> key stmt_op value
stmt_op -> "=" | "<" | ">" |"<=" | ">="
