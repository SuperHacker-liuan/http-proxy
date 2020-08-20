use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum HttpProxyError {
        Io(err: std::io::Error) {
            from()
        }

        HttpParse(err: httparse::Error) {
            from()
        }

        FromUtf8(err: std::string::FromUtf8Error) {
            from()
        }
    }
}
