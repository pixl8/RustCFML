// Java shim handlers - to be inserted into lib.rs

use cfml_common::dynamic::CfmlValue;
use cfml_common::vm::CfmlResult;
use indexmap::IndexMap;

pub fn handle_java_messagedigest(
    method: &str,
    args: Vec<CfmlValue>,
    object: &CfmlValue,
) -> CfmlResult {
    match method {
        "init" | "getinstance" => {
            let algorithm = args
                .first()
                .map(|a| a.as_string().to_lowercase())
                .unwrap_or_else(|| "sha-256".to_string());
            let mut shim = IndexMap::new();
            shim.insert(
                "__java_class".to_string(),
                CfmlValue::String("java.security.messagedigest".to_string()),
            );
            shim.insert("__java_shim".to_string(), CfmlValue::Bool(true));
            shim.insert("__algorithm".to_string(), CfmlValue::String(algorithm));
            shim.insert("__data".to_string(), CfmlValue::String(String::new()));
            Ok(CfmlValue::Struct(shim))
        }
        "update" => {
            // Real Java MessageDigest.update takes a byte[]. We accept both
            // Binary (from "...".getBytes()) and String (lenient) so Lucee and
            // RustCFML run the same interop code without rewrites.
            if let CfmlValue::Struct(ref shim) = object {
                let current = shim
                    .get("__data")
                    .map(|d| d.as_string())
                    .unwrap_or_default();
                let input = match args.first() {
                    Some(CfmlValue::Binary(b)) => String::from_utf8_lossy(b).to_string(),
                    Some(v) => v.as_string(),
                    None => String::new(),
                };
                let mut new_shim = shim.clone();
                new_shim.insert(
                    "__data".to_string(),
                    CfmlValue::String(format!("{}{}", current, input)),
                );
                Ok(CfmlValue::Struct(new_shim))
            } else {
                Ok(CfmlValue::Null)
            }
        }
        "digest" => {
            if let CfmlValue::Struct(ref shim) = object {
                let data = shim
                    .get("__data")
                    .map(|d| d.as_string())
                    .unwrap_or_default();
                Ok(CfmlValue::Binary(data.into_bytes()))
            } else {
                Ok(CfmlValue::Null)
            }
        }
        "isequal" => {
            if args.len() >= 2 {
                Ok(CfmlValue::Bool(args[0].as_string() == args[1].as_string()))
            } else {
                Ok(CfmlValue::Null)
            }
        }
        "reset" => {
            if let CfmlValue::Struct(ref shim) = object {
                let mut new_shim = shim.clone();
                new_shim.insert("__data".to_string(), CfmlValue::String(String::new()));
                Ok(CfmlValue::Struct(new_shim))
            } else {
                Ok(CfmlValue::Null)
            }
        }
        _ => Ok(CfmlValue::Null),
    }
}

pub fn handle_java_uuid(method: &str, _args: Vec<CfmlValue>, object: &CfmlValue) -> CfmlResult {
    match method {
        "init" | "randomuuid" => {
            let uuid = format!("{:032x}", rand_u128());
            let mut shim = IndexMap::new();
            shim.insert(
                "__java_class".to_string(),
                CfmlValue::String("java.util.uuid".to_string()),
            );
            shim.insert("__java_shim".to_string(), CfmlValue::Bool(true));
            shim.insert("__uuid".to_string(), CfmlValue::String(uuid));
            Ok(CfmlValue::Struct(shim))
        }
        "tostring" => {
            if let CfmlValue::Struct(ref shim) = object {
                if let Some(CfmlValue::String(uuid)) = shim.get("__uuid") {
                    if uuid.len() >= 32 {
                        let formatted = format!(
                            "{}-{}-{}-{}-{}",
                            &uuid[0..8],
                            &uuid[8..12],
                            &uuid[12..16],
                            &uuid[16..20],
                            &uuid[20..32]
                        );
                        return Ok(CfmlValue::String(formatted));
                    }
                }
            }
            Ok(CfmlValue::Null)
        }
        "getversion" => Ok(CfmlValue::Int(4)),
        "getvariant" => Ok(CfmlValue::Int(2)),
        _ => Ok(CfmlValue::Null),
    }
}

pub fn handle_java_thread(method: &str, _args: Vec<CfmlValue>, object: &CfmlValue) -> CfmlResult {
    // "threadgroup" is a nested shim for java.lang.ThreadGroup accessed via
    // Thread.getThreadGroup(). We route its own methods here too.
    if let CfmlValue::Struct(ref shim) = object {
        if shim
            .get("__java_class")
            .map(|v| v.as_string())
            .unwrap_or_default()
            == "java.lang.threadgroup"
        {
            return match method {
                "getname" => Ok(shim
                    .get("__name")
                    .cloned()
                    .unwrap_or(CfmlValue::String("main".to_string()))),
                _ => Ok(CfmlValue::Null),
            };
        }
    }
    match method {
        "init" | "currentthread" => {
            let mut shim = IndexMap::new();
            shim.insert(
                "__java_class".to_string(),
                CfmlValue::String("java.lang.thread".to_string()),
            );
            shim.insert("__java_shim".to_string(), CfmlValue::Bool(true));
            shim.insert("__name".to_string(), CfmlValue::String("main".to_string()));
            Ok(CfmlValue::Struct(shim))
        }
        "getname" => {
            if let CfmlValue::Struct(ref shim) = object {
                Ok(shim
                    .get("__name")
                    .cloned()
                    .unwrap_or(CfmlValue::String("main".to_string())))
            } else {
                Ok(CfmlValue::String("main".to_string()))
            }
        }
        "getthreadgroup" => {
            let mut tg = IndexMap::new();
            tg.insert(
                "__java_class".to_string(),
                CfmlValue::String("java.lang.threadgroup".to_string()),
            );
            tg.insert("__java_shim".to_string(), CfmlValue::Bool(true));
            tg.insert("__name".to_string(), CfmlValue::String("main".to_string()));
            Ok(CfmlValue::Struct(tg))
        }
        "getpriority" => Ok(CfmlValue::Int(5)),
        "isdaemon" => Ok(CfmlValue::Bool(false)),
        "sleep" => Ok(CfmlValue::Null),
        _ => Ok(CfmlValue::Null),
    }
}

pub fn handle_java_inetaddress(
    method: &str,
    args: Vec<CfmlValue>,
    object: &CfmlValue,
) -> CfmlResult {
    match method {
        "getlocalhost" => {
            let hostname = std::env::var("HOSTNAME")
                .or_else(|_| std::env::var("HOST"))
                .unwrap_or_else(|_| "localhost".to_string());
            let mut shim = IndexMap::new();
            shim.insert(
                "__java_class".to_string(),
                CfmlValue::String("java.net.inetaddress".to_string()),
            );
            shim.insert("__java_shim".to_string(), CfmlValue::Bool(true));
            shim.insert(
                "__hostname".to_string(),
                CfmlValue::String(hostname.clone()),
            );
            shim.insert(
                "__address".to_string(),
                CfmlValue::String("127.0.0.1".to_string()),
            );
            Ok(CfmlValue::Struct(shim))
        }
        "getbyname" => {
            let hostname = args
                .first()
                .map(|a| a.as_string())
                .unwrap_or_else(|| "localhost".to_string());
            let mut shim = IndexMap::new();
            shim.insert(
                "__java_class".to_string(),
                CfmlValue::String("java.net.inetaddress".to_string()),
            );
            shim.insert("__java_shim".to_string(), CfmlValue::Bool(true));
            shim.insert(
                "__hostname".to_string(),
                CfmlValue::String(hostname.clone()),
            );
            shim.insert(
                "__address".to_string(),
                CfmlValue::String("127.0.0.1".to_string()),
            );
            Ok(CfmlValue::Struct(shim))
        }
        "gethostname" | "gethostaddress" | "getcanonicalhostname" | "tostring" => {
            if let CfmlValue::Struct(ref shim) = object {
                let key = match method {
                    "gethostname" | "tostring" => "__hostname",
                    "gethostaddress" => "__address",
                    _ => "__hostname",
                };
                Ok(shim
                    .get(key)
                    .cloned()
                    .unwrap_or(CfmlValue::String("localhost".to_string())))
            } else {
                Ok(CfmlValue::String("localhost".to_string()))
            }
        }
        _ => Ok(CfmlValue::Null),
    }
}

pub fn handle_java_file(method: &str, args: Vec<CfmlValue>, object: &CfmlValue) -> CfmlResult {
    match method {
        "init" => {
            let path = args.first().map(|a| a.as_string()).unwrap_or_default();
            let mut shim = IndexMap::new();
            shim.insert(
                "__java_class".to_string(),
                CfmlValue::String("java.io.file".to_string()),
            );
            shim.insert("__java_shim".to_string(), CfmlValue::Bool(true));
            shim.insert("__path".to_string(), CfmlValue::String(path));
            Ok(CfmlValue::Struct(shim))
        }
        "tostring" => {
            // java.io.File.toString() returns the original path as given.
            if let CfmlValue::Struct(ref shim) = object {
                return Ok(shim
                    .get("__path")
                    .cloned()
                    .unwrap_or(CfmlValue::String(String::new())));
            }
            Ok(CfmlValue::String(String::new()))
        }
        "getabsolute_path" | "getabsolutepath" | "getcanonicalpath" => {
            if let CfmlValue::Struct(ref shim) = object {
                if let Some(CfmlValue::String(path)) = shim.get("__path") {
                    let p = std::path::Path::new(path);
                    if p.is_absolute() {
                        return Ok(CfmlValue::String(path.clone()));
                    }
                    if let Ok(cwd) = std::env::current_dir() {
                        return Ok(CfmlValue::String(
                            cwd.join(path).to_string_lossy().to_string(),
                        ));
                    }
                }
            }
            Ok(CfmlValue::String(String::new()))
        }
        "isabsolute" => {
            if let CfmlValue::Struct(ref shim) = object {
                if let Some(CfmlValue::String(path)) = shim.get("__path") {
                    return Ok(CfmlValue::Bool(std::path::Path::new(path).is_absolute()));
                }
            }
            Ok(CfmlValue::Bool(false))
        }
        "exists" => {
            if let CfmlValue::Struct(ref shim) = object {
                if let Some(CfmlValue::String(path)) = shim.get("__path") {
                    return Ok(CfmlValue::Bool(std::path::Path::new(path).exists()));
                }
            }
            Ok(CfmlValue::Bool(false))
        }
        "isfile" | "is_directory" | "isdirectory" => {
            if let CfmlValue::Struct(ref shim) = object {
                if let Some(CfmlValue::String(path)) = shim.get("__path") {
                    let p = std::path::Path::new(path);
                    return Ok(CfmlValue::Bool(if method == "isfile" {
                        p.is_file()
                    } else {
                        p.is_dir()
                    }));
                }
            }
            Ok(CfmlValue::Bool(false))
        }
        "getname" | "lastmodified" | "length" => {
            if let CfmlValue::Struct(ref shim) = object {
                if let Some(CfmlValue::String(path)) = shim.get("__path") {
                    if let Ok(meta) = std::fs::metadata(path) {
                        if method == "getname" {
                            if let Some(n) = std::path::Path::new(path).file_name() {
                                return Ok(CfmlValue::String(n.to_string_lossy().to_string()));
                            }
                        } else if method == "lastmodified" {
                            if let Ok(t) = meta.modified() {
                                let d = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
                                return Ok(CfmlValue::Double(d.as_millis() as f64));
                            }
                        } else {
                            return Ok(CfmlValue::Int(meta.len() as i64));
                        }
                    }
                }
            }
            Ok(CfmlValue::Int(0))
        }
        "topath" => {
            // File.toPath() returns a java.nio.file.Path. This is the portable
            // alternative to Paths.get(…), which Lucee can't dispatch to
            // cleanly due to its String/varargs signature.
            if let CfmlValue::Struct(ref shim) = object {
                if let Some(path) = shim.get("__path") {
                    let mut ps = IndexMap::new();
                    ps.insert(
                        "__java_class".to_string(),
                        CfmlValue::String("java.nio.file.paths".to_string()),
                    );
                    ps.insert("__java_shim".to_string(), CfmlValue::Bool(true));
                    ps.insert("__path".to_string(), path.clone());
                    return Ok(CfmlValue::Struct(ps));
                }
            }
            Ok(CfmlValue::Null)
        }
        _ => Ok(CfmlValue::Null),
    }
}

pub fn handle_java_system(method: &str, args: Vec<CfmlValue>, _object: &CfmlValue) -> CfmlResult {
    match method {
        "init" => {
            // java.lang.System is a static-only class in real Java, but we
            // return a shim struct so both init() and static-style access work.
            let mut shim = IndexMap::new();
            shim.insert(
                "__java_class".to_string(),
                CfmlValue::String("java.lang.system".to_string()),
            );
            shim.insert("__java_shim".to_string(), CfmlValue::Bool(true));
            // Expose `out` as a nested shim so `system.out.println(...)` works.
            let mut out = IndexMap::new();
            out.insert(
                "__java_class".to_string(),
                CfmlValue::String("java.lang.system.out".to_string()),
            );
            out.insert("__java_shim".to_string(), CfmlValue::Bool(true));
            shim.insert("out".to_string(), CfmlValue::Struct(out));
            Ok(CfmlValue::Struct(shim))
        }
        "currenttimemillis" => {
            let n = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as f64)
                .unwrap_or(0.0);
            Ok(CfmlValue::Double(n))
        }
        "nanotime" => {
            let n = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as f64)
                .unwrap_or(0.0);
            Ok(CfmlValue::Double(n))
        }
        "getproperty" => {
            // Some callers pass the key as the first "real" arg, but member
            // dispatch prepends the object — skip leading shim structs.
            let key = args
                .iter()
                .find_map(|a| match a {
                    CfmlValue::String(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();
            let val = match key.to_lowercase().as_str() {
                "os.name" => std::env::consts::OS.to_string(),
                "file.separator" => std::path::MAIN_SEPARATOR.to_string(),
                "path.separator" => {
                    if cfg!(unix) {
                        ":".to_string()
                    } else {
                        ";".to_string()
                    }
                }
                "user.dir" => std::env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default(),
                "user.home" => std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_default(),
                "java.version" => "rustcfml".to_string(),
                _ => String::new(),
            };
            Ok(CfmlValue::String(val))
        }
        "getenv" => {
            // No-arg form returns a struct of all env vars (real Java returns a Map).
            // Single-arg form returns the value for that key.
            let key = args.iter().find_map(|a| match a {
                CfmlValue::String(s) => Some(s.clone()),
                _ => None,
            });
            match key {
                Some(k) => Ok(CfmlValue::String(std::env::var(&k).unwrap_or_default())),
                None => {
                    let mut env = IndexMap::new();
                    for (k, v) in std::env::vars() {
                        env.insert(k, CfmlValue::String(v));
                    }
                    Ok(CfmlValue::Struct(env))
                }
            }
        }
        _ => Ok(CfmlValue::Null),
    }
}

pub fn handle_java_stringbuilder(
    method: &str,
    args: Vec<CfmlValue>,
    object: &CfmlValue,
) -> CfmlResult {
    match method {
        "init" => {
            let init = args.first().map(|a| a.as_string()).unwrap_or_default();
            let mut shim = IndexMap::new();
            shim.insert(
                "__java_class".to_string(),
                CfmlValue::String("java.lang.stringbuilder".to_string()),
            );
            shim.insert("__java_shim".to_string(), CfmlValue::Bool(true));
            shim.insert("__buffer".to_string(), CfmlValue::String(init));
            Ok(CfmlValue::Struct(shim))
        }
        "append" => {
            if let CfmlValue::Struct(ref shim) = object {
                let cur = shim
                    .get("__buffer")
                    .map(|b| b.as_string())
                    .unwrap_or_default();
                let app = args.first().map(|a| a.as_string()).unwrap_or_default();
                let mut ns = shim.clone();
                ns.insert(
                    "__buffer".to_string(),
                    CfmlValue::String(format!("{}{}", cur, app)),
                );
                Ok(CfmlValue::Struct(ns))
            } else {
                Ok(CfmlValue::Null)
            }
        }
        "tostring" => {
            if let CfmlValue::Struct(ref shim) = object {
                Ok(shim
                    .get("__buffer")
                    .cloned()
                    .unwrap_or(CfmlValue::String(String::new())))
            } else {
                Ok(CfmlValue::String(String::new()))
            }
        }
        "length" => {
            if let CfmlValue::Struct(ref shim) = object {
                let b = shim
                    .get("__buffer")
                    .map(|x| x.as_string())
                    .unwrap_or_default();
                Ok(CfmlValue::Int(b.len() as i64))
            } else {
                Ok(CfmlValue::Int(0))
            }
        }
        "clear" => {
            if let CfmlValue::Struct(ref shim) = object {
                let mut ns = shim.clone();
                ns.insert("__buffer".to_string(), CfmlValue::String(String::new()));
                Ok(CfmlValue::Struct(ns))
            } else {
                Ok(CfmlValue::Null)
            }
        }
        _ => Ok(CfmlValue::Null),
    }
}

// ---- TreeMap ----
pub fn handle_java_treemap(method: &str, args: Vec<CfmlValue>, object: &CfmlValue) -> CfmlResult {
    match method {
        "init" => {
            let mut shim = IndexMap::new();
            shim.insert(
                "__java_class".to_string(),
                CfmlValue::String("java.util.treemap".to_string()),
            );
            shim.insert("__java_shim".to_string(), CfmlValue::Bool(true));
            if let Some(CfmlValue::Struct(init)) = args.first() {
                for (k, v) in init {
                    shim.insert(k.clone(), v.clone());
                }
            }
            Ok(CfmlValue::Struct(shim))
        }
        "put" => {
            if let CfmlValue::Struct(ref shim) = object {
                if let Some((k, v)) = args.get(0).zip(args.get(1)) {
                    let mut ns = shim.clone();
                    ns.insert(k.as_string(), v.clone());
                    Ok(CfmlValue::Struct(ns))
                } else {
                    Ok(object.clone())
                }
            } else {
                Ok(CfmlValue::Null)
            }
        }
        "keyset" | "keys" => {
            if let CfmlValue::Struct(ref shim) = object {
                let mut ks: Vec<String> = shim
                    .iter()
                    .filter(|(k, _)| !k.starts_with("__"))
                    .map(|(k, _)| k.clone())
                    .collect();
                ks.sort(); // TreeMap = sorted key order
                Ok(CfmlValue::Array(
                    ks.into_iter().map(CfmlValue::String).collect(),
                ))
            } else {
                Ok(CfmlValue::Array(Vec::new()))
            }
        }
        "get" => {
            if let CfmlValue::Struct(ref shim) = object {
                if let Some(key) = args.first() {
                    let k = key.as_string();
                    return Ok(shim.get(&k).cloned().unwrap_or(CfmlValue::Null));
                }
            }
            Ok(CfmlValue::Null)
        }
        "size" | "len" => {
            if let CfmlValue::Struct(ref shim) = object {
                Ok(CfmlValue::Int(
                    shim.iter().filter(|(k, _)| !k.starts_with("__")).count() as i64,
                ))
            } else {
                Ok(CfmlValue::Int(0))
            }
        }
        "containskey" => {
            if let CfmlValue::Struct(ref shim) = object {
                if let Some(key) = args.first() {
                    let k = key.as_string();
                    return Ok(CfmlValue::Bool(shim.contains_key(&k)));
                }
            }
            Ok(CfmlValue::Bool(false))
        }
        "isempty" => {
            if let CfmlValue::Struct(ref shim) = object {
                Ok(CfmlValue::Bool(
                    shim.iter().all(|(k, _)| k.starts_with("__")),
                ))
            } else {
                Ok(CfmlValue::Bool(true))
            }
        }
        _ => Ok(CfmlValue::Null),
    }
}

pub fn handle_java_linkedhashmap(
    method: &str,
    args: Vec<CfmlValue>,
    object: &CfmlValue,
) -> CfmlResult {
    match method {
        "init" => {
            let mut shim = IndexMap::new();
            shim.insert(
                "__java_class".to_string(),
                CfmlValue::String("java.util.linkedhashmap".to_string()),
            );
            shim.insert("__java_shim".to_string(), CfmlValue::Bool(true));
            Ok(CfmlValue::Struct(shim))
        }
        "keyset" | "keys" => {
            if let CfmlValue::Struct(ref shim) = object {
                let ks: Vec<CfmlValue> = shim
                    .iter()
                    .filter(|(k, _)| !k.starts_with("__"))
                    .map(|(k, _)| CfmlValue::String(k.clone()))
                    .collect();
                Ok(CfmlValue::Array(ks))
            } else {
                Ok(CfmlValue::Array(Vec::new()))
            }
        }
        "get" => {
            if let CfmlValue::Struct(ref shim) = object {
                if let Some(CfmlValue::String(k)) = args.first() {
                    Ok(shim.get(k).cloned().unwrap_or(CfmlValue::Null))
                } else {
                    Ok(CfmlValue::Null)
                }
            } else {
                Ok(CfmlValue::Null)
            }
        }
        "size" | "len" => {
            if let CfmlValue::Struct(ref shim) = object {
                Ok(CfmlValue::Int(
                    shim.iter().filter(|(k, _)| !k.starts_with("__")).count() as i64,
                ))
            } else {
                Ok(CfmlValue::Int(0))
            }
        }
        "containskey" => {
            if let CfmlValue::Struct(ref shim) = object {
                if let Some(CfmlValue::String(k)) = args.first() {
                    Ok(CfmlValue::Bool(shim.contains_key(k)))
                } else {
                    Ok(CfmlValue::Bool(false))
                }
            } else {
                Ok(CfmlValue::Bool(false))
            }
        }
        "put" => {
            if let CfmlValue::Struct(ref shim) = object {
                if let Some((k, v)) = args.get(0).zip(args.get(1)) {
                    let mut ns = shim.clone();
                    ns.insert(k.as_string(), v.clone());
                    Ok(CfmlValue::Struct(ns))
                } else {
                    Ok(object.clone())
                }
            } else {
                Ok(CfmlValue::Null)
            }
        }
        "isempty" => {
            if let CfmlValue::Struct(ref shim) = object {
                Ok(CfmlValue::Bool(
                    shim.iter().all(|(k, _)| k.starts_with("__")),
                ))
            } else {
                Ok(CfmlValue::Bool(true))
            }
        }
        _ => Ok(CfmlValue::Null),
    }
}

pub fn handle_java_concurrentlinkedqueue(
    method: &str,
    args: Vec<CfmlValue>,
    object: &CfmlValue,
) -> CfmlResult {
    match method {
        "init" => {
            let mut shim = IndexMap::new();
            shim.insert(
                "__java_class".to_string(),
                CfmlValue::String("java.util.concurrent.concurrentlinkedqueue".to_string()),
            );
            shim.insert("__java_shim".to_string(), CfmlValue::Bool(true));
            shim.insert("__queue".to_string(), CfmlValue::Array(Vec::new()));
            Ok(CfmlValue::Struct(shim))
        }
        "offer" => {
            if let CfmlValue::Struct(ref shim) = object {
                if let Some(item) = args.first() {
                    let mut ns = shim.clone();
                    if let Some(CfmlValue::Array(q)) = ns.get("__queue").cloned() {
                        let mut nq = q.clone();
                        nq.push(item.clone());
                        ns.insert("__queue".to_string(), CfmlValue::Array(nq));
                    }
                    Ok(CfmlValue::Struct(ns))
                } else {
                    Ok(object.clone())
                }
            } else {
                Ok(CfmlValue::Null)
            }
        }
        "poll" => {
            if let CfmlValue::Struct(ref shim) = object {
                if let Some(CfmlValue::Array(q)) = shim.get("__queue").cloned() {
                    if !q.is_empty() {
                        let mut ns = shim.clone();
                        let itm = q[0].clone();
                        let mut nq = q[1..].to_vec();
                        ns.insert("__queue".to_string(), CfmlValue::Array(nq));
                        return Ok(CfmlValue::Struct(ns));
                    }
                }
                Ok(CfmlValue::Null)
            } else {
                Ok(CfmlValue::Null)
            }
        }
        "peek" => {
            if let CfmlValue::Struct(ref shim) = object {
                if let Some(CfmlValue::Array(q)) = shim.get("__queue").cloned() {
                    if !q.is_empty() {
                        return Ok(q[0].clone());
                    }
                }
                Ok(CfmlValue::Null)
            } else {
                Ok(CfmlValue::Null)
            }
        }
        "size" | "len" => {
            if let CfmlValue::Struct(ref shim) = object {
                if let Some(CfmlValue::Array(q)) = shim.get("__queue").cloned() {
                    Ok(CfmlValue::Int(q.len() as i64))
                } else {
                    Ok(CfmlValue::Int(0))
                }
            } else {
                Ok(CfmlValue::Int(0))
            }
        }
        "isempty" => {
            if let CfmlValue::Struct(ref shim) = object {
                if let Some(CfmlValue::Array(q)) = shim.get("__queue").cloned() {
                    return Ok(CfmlValue::Bool(q.is_empty()));
                }
                Ok(CfmlValue::Bool(true))
            } else {
                Ok(CfmlValue::Bool(true))
            }
        }
        _ => Ok(CfmlValue::Null),
    }
}

pub fn handle_java_paths(method: &str, args: Vec<CfmlValue>, object: &CfmlValue) -> CfmlResult {
    match method {
        "init" => {
            // Paths is a static-only class; return a stub shim so that
            // the subsequent .get(path) static call dispatches here.
            let mut shim = IndexMap::new();
            shim.insert(
                "__java_class".to_string(),
                CfmlValue::String("java.nio.file.paths".to_string()),
            );
            shim.insert("__java_shim".to_string(), CfmlValue::Bool(true));
            Ok(CfmlValue::Struct(shim))
        }
        "get" => {
            let path = args.first().map(|a| a.as_string()).unwrap_or_default();
            let mut shim = IndexMap::new();
            shim.insert(
                "__java_class".to_string(),
                CfmlValue::String("java.nio.file.paths".to_string()),
            );
            shim.insert("__java_shim".to_string(), CfmlValue::Bool(true));
            shim.insert("__path".to_string(), CfmlValue::String(path));
            Ok(CfmlValue::Struct(shim))
        }
        "getparent" => {
            if let CfmlValue::Struct(ref shim) = object {
                if let Some(CfmlValue::String(path)) = shim.get("__path") {
                    if let Some(p) = std::path::Path::new(path).parent() {
                        let mut ps = IndexMap::new();
                        ps.insert(
                            "__java_class".to_string(),
                            CfmlValue::String("java.nio.file.paths".to_string()),
                        );
                        ps.insert("__java_shim".to_string(), CfmlValue::Bool(true));
                        ps.insert(
                            "__path".to_string(),
                            CfmlValue::String(p.to_string_lossy().to_string()),
                        );
                        return Ok(CfmlValue::Struct(ps));
                    }
                }
                Ok(CfmlValue::Null)
            } else {
                Ok(CfmlValue::Null)
            }
        }
        "isabsolute" => {
            if let CfmlValue::Struct(ref shim) = object {
                if let Some(CfmlValue::String(path)) = shim.get("__path") {
                    return Ok(CfmlValue::Bool(std::path::Path::new(path).is_absolute()));
                }
                Ok(CfmlValue::Bool(false))
            } else {
                Ok(CfmlValue::Bool(false))
            }
        }
        "tostring" => {
            if let CfmlValue::Struct(ref shim) = object {
                Ok(shim
                    .get("__path")
                    .cloned()
                    .unwrap_or(CfmlValue::String(String::new())))
            } else {
                Ok(CfmlValue::String(String::new()))
            }
        }
        "toabsolute" | "toabsolutepath" => {
            if let CfmlValue::Struct(ref shim) = object {
                if let Some(CfmlValue::String(path)) = shim.get("__path") {
                    let p = std::path::Path::new(path);
                    if p.is_absolute() {
                        return Ok(shim.get("__path").cloned().unwrap_or(CfmlValue::Null));
                    }
                    if let Ok(cwd) = std::env::current_dir() {
                        let full = cwd.join(path);
                        let mut ns = shim.clone();
                        ns.insert(
                            "__path".to_string(),
                            CfmlValue::String(full.to_string_lossy().to_string()),
                        );
                        return Ok(CfmlValue::Struct(ns));
                    }
                }
                Ok(CfmlValue::Null)
            } else {
                Ok(CfmlValue::Null)
            }
        }
        _ => Ok(CfmlValue::Null),
    }
}

fn rand_u128() -> u128 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;
    let mut h = DefaultHasher::new();
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .hash(&mut h);
    0x12345678u64.hash(&mut h);
    h.finish() as u128
}
