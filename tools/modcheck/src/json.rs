//! Just enough JSON for `mod.json`: the whole grammar, no serde (the workspace takes no crates.io
//! dependencies yet, see the T5 journal).

#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    /// Keys in file order; a repeated key is refused by the parser.
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(kv) => kv.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn num(&self) -> Option<f64> {
        match self {
            Json::Num(n) => Some(*n),
            _ => None,
        }
    }

    pub fn arr(&self) -> Option<&[Json]> {
        match self {
            Json::Arr(a) => Some(a),
            _ => None,
        }
    }

    pub fn parse(text: &str) -> Result<Json, String> {
        let mut p = Parser { s: text.as_bytes(), at: 0 };
        let v = p.value()?;
        p.ws();
        if p.at != p.s.len() {
            return Err(p.err("text after the value"));
        }
        Ok(v)
    }
}

struct Parser<'a> {
    s: &'a [u8],
    at: usize,
}

impl Parser<'_> {
    fn err(&self, what: &str) -> String {
        let line = self.s[..self.at.min(self.s.len())].iter().filter(|&&b| b == b'\n').count() + 1;
        format!("line {line}: {what}")
    }

    fn ws(&mut self) {
        while self.at < self.s.len() && matches!(self.s[self.at], b' ' | b'\t' | b'\r' | b'\n') {
            self.at += 1;
        }
    }

    fn eat(&mut self, lit: &str) -> bool {
        if self.s[self.at..].starts_with(lit.as_bytes()) {
            self.at += lit.len();
            true
        } else {
            false
        }
    }

    fn value(&mut self) -> Result<Json, String> {
        self.ws();
        match self.s.get(self.at) {
            None => Err(self.err("unexpected end")),
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => self.string().map(Json::Str),
            Some(b't') if self.eat("true") => Ok(Json::Bool(true)),
            Some(b'f') if self.eat("false") => Ok(Json::Bool(false)),
            Some(b'n') if self.eat("null") => Ok(Json::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            Some(_) => Err(self.err("not a JSON value")),
        }
    }

    fn object(&mut self) -> Result<Json, String> {
        self.at += 1;
        let mut kv: Vec<(String, Json)> = Vec::new();
        self.ws();
        if self.eat("}") {
            return Ok(Json::Obj(kv));
        }
        loop {
            self.ws();
            if self.s.get(self.at) != Some(&b'"') {
                return Err(self.err("an object key must be a string"));
            }
            let k = self.string()?;
            if kv.iter().any(|(have, _)| *have == k) {
                return Err(self.err(&format!("key {k:?} given twice")));
            }
            self.ws();
            if !self.eat(":") {
                return Err(self.err("expected ':'"));
            }
            let v = self.value()?;
            kv.push((k, v));
            self.ws();
            if self.eat(",") {
                continue;
            }
            if self.eat("}") {
                return Ok(Json::Obj(kv));
            }
            return Err(self.err("expected ',' or '}'"));
        }
    }

    fn array(&mut self) -> Result<Json, String> {
        self.at += 1;
        let mut a = Vec::new();
        self.ws();
        if self.eat("]") {
            return Ok(Json::Arr(a));
        }
        loop {
            a.push(self.value()?);
            self.ws();
            if self.eat(",") {
                continue;
            }
            if self.eat("]") {
                return Ok(Json::Arr(a));
            }
            return Err(self.err("expected ',' or ']'"));
        }
    }

    fn hex4(&mut self) -> Result<u32, String> {
        let h = self.s.get(self.at..self.at + 4).ok_or_else(|| self.err("short \\u escape"))?;
        let h = std::str::from_utf8(h).map_err(|_| self.err("bad \\u escape"))?;
        let v = u32::from_str_radix(h, 16).map_err(|_| self.err("bad \\u escape"))?;
        self.at += 4;
        Ok(v)
    }

    fn string(&mut self) -> Result<String, String> {
        self.at += 1;
        let mut out = Vec::new();
        loop {
            let Some(&c) = self.s.get(self.at) else { return Err(self.err("unterminated string")) };
            self.at += 1;
            match c {
                b'"' => break,
                b'\\' => {
                    let Some(&e) = self.s.get(self.at) else { return Err(self.err("bad escape")) };
                    self.at += 1;
                    let ch = match e {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'/' => '/',
                        b'b' => '\u{8}',
                        b'f' => '\u{c}',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'u' => {
                            let hi = self.hex4()?;
                            let cp = if (0xd800..0xdc00).contains(&hi) && self.eat("\\u") {
                                let lo = self.hex4()?;
                                0x10000 + ((hi - 0xd800) << 10) + (lo.wrapping_sub(0xdc00) & 0x3ff)
                            } else {
                                hi
                            };
                            char::from_u32(cp).ok_or_else(|| self.err("bad \\u escape"))?
                        }
                        _ => return Err(self.err("bad escape")),
                    };
                    let mut buf = [0u8; 4];
                    out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                }
                0..0x20 => return Err(self.err("control character in a string")),
                _ => out.push(c),
            }
        }
        String::from_utf8(out).map_err(|_| self.err("string is not UTF-8"))
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.at;
        while self.at < self.s.len() && matches!(self.s[self.at], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9') {
            self.at += 1;
        }
        let t = std::str::from_utf8(&self.s[start..self.at]).expect("ascii");
        t.parse::<f64>().map(Json::Num).map_err(|_| self.err(&format!("{t:?} is not a number")))
    }
}

#[cfg(test)]
mod tests {
    use super::Json;

    #[test]
    fn reads_the_standards_example() {
        let j = Json::parse(
            r#"{"standard": 1, "source": {"members": ["Env\\MC01\\mc01.res"]}, "todo": [], "x": [-1.5e2, true, null, "é"]}"#,
        )
        .unwrap();
        assert_eq!(j.get("standard").and_then(Json::num), Some(1.0));
        let m = j.get("source").and_then(|s| s.get("members")).and_then(Json::arr).unwrap();
        assert_eq!(m[0].str(), Some(r"Env\MC01\mc01.res"));
        assert_eq!(j.get("todo"), Some(&Json::Arr(vec![])));
        assert_eq!(
            j.get("x"),
            Some(&Json::Arr(vec![Json::Num(-150.0), Json::Bool(true), Json::Null, Json::Str("é".into())]))
        );
    }

    #[test]
    fn refuses_what_is_not_json() {
        for bad in ["{", "{\"a\":1,}", "[1 2]", "{\"a\":1,\"a\":2}", "{'a':1}", "1 2", "\"\n\""] {
            assert!(Json::parse(bad).is_err(), "{bad:?} parsed");
        }
    }
}
