use std::collections::HashMap;

pub struct Response<'a> {
    pub status: &'a [u8],
    pub status_code: &'a [u8],
    pub http_version: &'a [u8],
    pub headers: HashMap<&'a [u8], &'a [u8]>,
    pub body: &'a [u8],
}

enum ResponseParseState {
    HttpVersion,
    StatusCode,
    Status,
    Headers { is_end: bool },
    Body,
}

pub fn parse_response(data: &[u8]) -> Response {
    let mut state = ResponseParseState::HttpVersion;
    let mut http_version = 0;
    let mut status_code = 0;
    let mut status = 0;
    let mut header = 0;
    let mut headers_key: Vec<usize> = vec![];
    let mut headers_value: Vec<usize> = vec![];
    for (i, current) in data.iter().enumerate() {
        match state {
            ResponseParseState::HttpVersion => {
                if current == &b' ' {
                    state = ResponseParseState::StatusCode;
                } else {
                    http_version = i;
                }
            }
            ResponseParseState::StatusCode => {
                if current == &b' ' {
                    state = ResponseParseState::Status;
                } else {
                    status_code = i;
                }
            }
            ResponseParseState::Status => {
                if current == &b'\n' {
                    state = ResponseParseState::Headers { is_end: false };
                } else {
                    status = i;
                }
            }
            ResponseParseState::Headers { is_end } => {
                if is_end {
                    if current == &b'\n' {
                        state = ResponseParseState::Body;
                    } else {
                        panic!("invalid state");
                    }
                } else if current == &b'\r' {
                    state = ResponseParseState::Headers { is_end: true };
                } else {
                    if current == &b'\n' {
                        headers_value.push(header);
                        header = 0;
                    } else if current == &b':' {
                        headers_key.push(header);
                        header = 0;
                    } else {
                        header = i;
                    }
                }
            }
            ResponseParseState::Body => {
                break;
            }
        }
    }

    let http_version_slice = &data[..=http_version];
    let status_code_slice = &data[http_version + 2..=status_code];
    let status_slice = &data[status_code + 2..=status];

    let mut headers = HashMap::new();
    let mut last = status + 2;
    for (key, value) in headers_key.iter().zip(headers_value) {
        let key_slice = &data[last..=*key];
        let value_slice = &data[key + 2..=value];
        last = value + 2;
        headers.insert(key_slice, value_slice);
    }

    let body_slice = &data[last + 2..];

    Response {
        status: status_slice,
        status_code: status_code_slice,
        http_version: http_version_slice,
        headers: headers,
        body: body_slice,
    }
}

pub struct Request<'a> {
    pub method: &'a [u8],
    pub url: &'a [u8],
    pub http_version: &'a [u8],
    pub headers: HashMap<&'a [u8], &'a [u8]>,
    pub body: &'a [u8],
}

enum RequestParseState {
    Method,
    Url,
    HttpVersion,
    Headers { is_end: bool },
    Body,
}

pub fn parse_request(data: &[u8]) -> Request {
    let mut state = RequestParseState::Method;
    let mut method = 0;
    let mut url = 0;
    let mut http_version = 0;
    let mut header = 0;
    let mut headers_key: Vec<usize> = vec![];
    let mut headers_value: Vec<usize> = vec![];
    for (i, current) in data.iter().enumerate() {
        match state {
            RequestParseState::Method => {
                if current == &b' ' {
                    state = RequestParseState::Url;
                } else {
                    method = i;
                }
            }
            RequestParseState::Url => {
                if current == &b' ' {
                    state = RequestParseState::HttpVersion;
                } else {
                    url = i;
                }
            }
            RequestParseState::HttpVersion => {
                if current == &b'\n' {
                    state = RequestParseState::Headers { is_end: false };
                } else {
                    http_version = i;
                }
            }
            RequestParseState::Headers { is_end } => {
                if is_end {
                    if current == &b'\n' {
                        state = RequestParseState::Body;
                    } else {
                        panic!("invalid state");
                    }
                } else if current == &b'\r' {
                    state = RequestParseState::Headers { is_end: true };
                } else {
                    if current == &b'\n' {
                        headers_value.push(header);
                        header = 0;
                    } else if current == &b':' {
                        headers_key.push(header);
                        header = 0;
                    } else {
                        header = i;
                    }
                }
            }
            RequestParseState::Body => {
                break;
            }
        }
    }

    let method_slice = &data[..=method];
    let url_slice = &data[method + 2..=url];
    let http_version_slice = &data[url + 2..=http_version];

    let mut headers = HashMap::new();
    let mut last = http_version + 2;
    for (key, value) in headers_key.iter().zip(headers_value) {
        let key_slice = &data[last..=*key];
        let value_slice = &data[key + 2..=value];
        last = value + 2;
        headers.insert(key_slice, value_slice);
    }

    let body_slice = &data[last + 2..];

    Request {
        method: method_slice,
        url: url_slice,
        http_version: http_version_slice,
        headers: headers,
        body: body_slice,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request() {
        let input = b"GET /index HTTP/1.1\nhost:test.com\nContent-Type:text/html\n\r\nabc";
        let result = parse_request(input);
        assert_eq!(result.method, b"GET");
        assert_eq!(result.url, b"/index");
        assert_eq!(result.http_version, b"HTTP/1.1");
        assert_eq!(result.headers.len(), 2);
        assert_eq!(result.headers.get(&b"host"[..]).unwrap(), &&b"test.com"[..]);
        assert_eq!(
            result.headers.get(&b"Content-Type"[..]).unwrap(),
            &&b"text/html"[..]
        );
        assert_eq!(result.body, b"abc");
    }

    #[test]
    fn test_parse_response() {
        let input = b"HTTP/1.1 200 OK\nContent-Length:88\nContent-Type:text/html\n\r\nbody123";
        let result = parse_response(input);
        assert_eq!(result.status_code, b"200");
        assert_eq!(result.status, b"OK");
        assert_eq!(result.http_version, b"HTTP/1.1");
        assert_eq!(result.headers.len(), 2);
        assert_eq!(
            result.headers.get(&b"Content-Length"[..]).unwrap(),
            &&b"88"[..]
        );
        assert_eq!(
            result.headers.get(&b"Content-Type"[..]).unwrap(),
            &&b"text/html"[..]
        );
        assert_eq!(result.body, b"body123");
    }
}
