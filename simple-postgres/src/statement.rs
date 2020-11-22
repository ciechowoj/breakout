
pub struct Statements<'a> {
    sql : &'a str
}

pub struct Statement {
    pub statement : String,
    pub params : Vec<u32>
}

pub fn split_statements(sql : &str) -> Statements {
    return Statements { sql };
}

impl<'a> Iterator for Statements<'a> {
    type Item = Statement;

    fn next(&mut self) -> Option<Self::Item> {
        self.sql = self.sql.trim_start();

        if self.sql != "" {
            let mut chars = self.sql.chars();

            let mut quotes : Option<&str> = None;
            let mut backslashes = 0u64;
            let mut comment_level = 0;
            let mut inline_comment = false;

            while let Some(ch) = chars.next() {
                if ch == ';' && quotes == None && comment_level == 0 && !inline_comment {
                    break;
                }
                else if !inline_comment && ch == '/' && chars.as_str().starts_with("*") { // [TODO] handle comments inside quotations
                    comment_level += 1;
                    chars = chars.as_str()[1..].chars();
                }
                else if !inline_comment && comment_level > 0 {
                    if ch == '*' && chars.as_str().starts_with("/") {
                        comment_level -= 1;
                        chars = chars.as_str()[1..].chars();
                    }
                }
                else if ch == '-' && chars.as_str().starts_with("-") {
                    inline_comment = true;
                    chars = chars.as_str()[1..].chars();
                }
                else if inline_comment {
                    if ch == '\n' || ch == '\r' {
                        inline_comment = false;
                    }
                }
                else if ch == '"' {
                    if quotes == None {
                        quotes = Some(&chars.as_str()[..1]);
                    }
                    else if quotes == Some("\"") && backslashes == 0 {
                        quotes = None;
                    }
                }
                else if ch == '\'' {
                    if quotes == None {
                        quotes = Some(&chars.as_str()[..1]);
                    }
                    else if quotes == Some("\'") && backslashes == 0 {
                        quotes = None;
                    }
                }
                else if ch == '$' {
                    if let Some(q) = quotes {
                        if q.starts_with("$") {
                            if let Some(stripped) = chars.as_str().strip_prefix(&q[1..]) {
                                chars = stripped.chars();
                                quotes = None;
                            }
                        }
                    }
                    else {
                        let mut nested = chars.clone();

                        if let Some(tg) = nested.next() {
                            if tg.is_digit(10) {
                                continue;
                            }
                            else if tg == '$' {
                                quotes = Some("$$");
                                continue;
                            }
                            else if !tg.is_alphabetic() {
                                continue;
                            }
                        }

                        while let Some(tg) = nested.next() {
                            if tg == '$' {
                                let begin = (chars.as_str().as_ptr() as usize) - (self.sql.as_ptr() as usize) - 1;
                                let end = (nested.as_str().as_ptr() as usize) - (self.sql.as_ptr() as usize);
                                quotes = Some(&self.sql[begin..end]);
                                chars = nested;
                                break;
                            }
                            else if !tg.is_alphanumeric() {
                                break;
                            }
                        }
                    }
                }

                if ch == '\\' {
                    backslashes = (backslashes + 1) % 2;
                }
                else {
                    backslashes = 0;
                }
            }

            let new_sql = chars.as_str();
            let result = &self.sql[..self.sql.len() - new_sql.len()];
            self.sql = new_sql;
            return Some(Statement { statement: result.to_owned(), params: vec!() });
        }

        return None;
    }
}

#[cfg(test)]
mod tests {
    use crate::statement::*;

    #[test]
    fn no_statements() {
        let statements : Vec<Statement> = split_statements("").collect();
        assert_eq!(0, statements.len());
        let statements : Vec<Statement> = split_statements(" \n\t  ").collect();
        assert_eq!(0, statements.len());
    }

    #[test]
    fn simple_splits() {
        let statements : Vec<Statement> = split_statements("SELECT * FROM foo;").collect();
        assert_eq!(1, statements.len());
        assert_eq!("SELECT * FROM foo;", statements[0].statement);

        let statements : Vec<Statement> = split_statements("SELECT * FROM foo; SELECT * FROM bar; CREATE TABLE qux;").collect();
        assert_eq!(3, statements.len());
        assert_eq!("SELECT * FROM foo;", statements[0].statement);
        assert_eq!("SELECT * FROM bar;", statements[1].statement);
        assert_eq!("CREATE TABLE qux;", statements[2].statement);
    }

    #[test]
    fn simple_splits_no_trailing_comma() {
        let statements : Vec<Statement> = split_statements("SELECT * FROM foo").collect();
        assert_eq!(1, statements.len());
        assert_eq!("SELECT * FROM foo", statements[0].statement);

        let statements : Vec<Statement> = split_statements("SELECT * FROM foo; SELECT * FROM bar; CREATE TABLE qux").collect();
        assert_eq!(3, statements.len());
        assert_eq!("SELECT * FROM foo;", statements[0].statement);
        assert_eq!("SELECT * FROM bar;", statements[1].statement);
        assert_eq!("CREATE TABLE qux", statements[2].statement);
    }

    #[test]
    fn statement_with_string() {
        let statements : Vec<Statement> = split_statements(r#"INSERT INTO table VALUES ("Hello; World;")"#).collect();
        assert_eq!(1, statements.len());
        assert_eq!(r#"INSERT INTO table VALUES ("Hello; World;")"#, statements[0].statement);

        let statements : Vec<Statement> = split_statements(r#"INSERT INTO table VALUES ("Hello;\" ; \\\" World;\\")"#).collect();
        assert_eq!(1, statements.len());
        assert_eq!(r#"INSERT INTO table VALUES ("Hello;\" ; \\\" World;\\")"#, statements[0].statement);

        let statements : Vec<Statement> = split_statements(r#"INSERT INTO table VALUES ('Hello; World;')"#).collect();
        assert_eq!(1, statements.len());
        assert_eq!(r#"INSERT INTO table VALUES ('Hello; World;')"#, statements[0].statement);

        let statements : Vec<Statement> = split_statements(r#"INSERT INTO table VALUES ('Hello;\' ; \\\' World;\\')"#).collect();
        assert_eq!(1, statements.len());
        assert_eq!(r#"INSERT INTO table VALUES ('Hello;\' ; \\\' World;\\')"#, statements[0].statement);

        let statements : Vec<Statement> = split_statements(r#"INSERT INTO table VALUES ('Hello;'' ; '' World;')"#).collect();
        assert_eq!(1, statements.len());
        assert_eq!(r#"INSERT INTO table VALUES ('Hello;'' ; '' World;')"#, statements[0].statement);
    }

    #[test]
    fn statement_with_dollar_quotation() {
        let statements : Vec<Statement> = split_statements(r#"INSERT INTO table VALUES ($$Hello; World;$$)"#).collect();
        assert_eq!(1, statements.len());
        assert_eq!(r#"INSERT INTO table VALUES ($$Hello; World;$$)"#, statements[0].statement);

        let statements : Vec<Statement> = split_statements(r#"INSERT INTO table VALUES ($tag$Hello; World;$tag$)"#).collect();
        assert_eq!(1, statements.len());
        assert_eq!(r#"INSERT INTO table VALUES ($tag$Hello; World;$tag$)"#, statements[0].statement);

        let statements : Vec<Statement> = split_statements(r#"INSERT INTO table VALUES ($tag$Hello; $$ World;$tag$)"#).collect();
        assert_eq!(1, statements.len());
        assert_eq!(r#"INSERT INTO table VALUES ($tag$Hello; $$ World;$tag$)"#, statements[0].statement);

        let statements : Vec<Statement> = split_statements(r#"INSERT INTO table VALUES ($tag$Hello; $tagged$ World;$tag$)"#).collect();
        assert_eq!(1, statements.len());
        assert_eq!(r#"INSERT INTO table VALUES ($tag$Hello; $tagged$ World;$tag$)"#, statements[0].statement);
    }

    #[test]
    fn statement_with_multiline_comments() {
        let statements : Vec<Statement> = split_statements(r#"INSERT INTO table VALUES /* ; */ ('xxx')"#).collect();
        assert_eq!(1, statements.len());
        assert_eq!(r#"INSERT INTO table VALUES /* ; */ ('xxx')"#, statements[0].statement);

        let statements : Vec<Statement> = split_statements(r#"INSERT INTO table VALUES /* ; /* ; */ ; */ ('xxx')"#).collect();
        assert_eq!(1, statements.len());
        assert_eq!(r#"INSERT INTO table VALUES /* ; /* ; */ ; */ ('xxx')"#, statements[0].statement);
    }

    #[test]
    fn statement_with_inline_comments() {
        let statements : Vec<Statement> = split_statements("INSERT INTO table VALUES -- ; \n ('xxx')").collect();
        assert_eq!(1, statements.len());
        assert_eq!("INSERT INTO table VALUES -- ; \n ('xxx')", statements[0].statement);

        let statements : Vec<Statement> = split_statements("INSERT INTO table VALUES -- something; more; \n ('xxx')").collect();
        assert_eq!(1, statements.len());
        assert_eq!("INSERT INTO table VALUES -- something; more; \n ('xxx')", statements[0].statement);

        let statements : Vec<Statement> = split_statements("INSERT INTO table VALUES \n--;\n ('xxx')").collect();
        assert_eq!(1, statements.len());
        assert_eq!("INSERT INTO table VALUES \n--;\n ('xxx')", statements[0].statement);

        let statements : Vec<Statement> = split_statements("INSERT INTO table VALUES \n--;\n--;\n ('xxx')").collect();
        assert_eq!(1, statements.len());
        assert_eq!("INSERT INTO table VALUES \n--;\n--;\n ('xxx')", statements[0].statement);
    }
}

