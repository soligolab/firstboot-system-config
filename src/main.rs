use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::thread;

const APP_HTML: &str = r#"<!doctype html><html lang='en'><head><meta charset='utf-8'/><meta name='viewport' content='width=device-width, initial-scale=1'/><title>First Boot</title>
<style>
:root{--bg:#0f1720;--panel:#1b2735;--panel2:#223245;--txt:#dce6f2;--muted:#9ab0c8;--ok:#2ecc95;--warn:#f0ba52;--bad:#df6161;--b:#40566f}
*{box-sizing:border-box}body{margin:0;font-family:Segoe UI,Arial;background:linear-gradient(#0c131b,#101926);color:var(--txt);min-height:600px}
.wrap{width:min(96vw,1500px);margin:14px auto}.panel{background:var(--panel);border:1px solid var(--b);border-radius:12px;padding:12px;box-shadow:0 8px 20px #0008}
.top,.btns{display:flex;gap:10px;align-items:center;flex-wrap:wrap}.sp{flex:1}h1{text-align:center;margin:10px 0 2px}.sub{text-align:center;color:var(--muted)}
.cards{display:grid;grid-template-columns:repeat(auto-fit,minmax(320px,1fr));gap:10px} .card{background:var(--panel2);border:1px solid var(--b);border-radius:12px;padding:10px}
label{display:block;margin:7px 0 4px;color:#c5d8eb}input,select,button{width:100%;padding:10px;border-radius:8px;border:1px solid var(--b);background:#111c27;color:var(--txt)}
button{cursor:pointer;background:linear-gradient(#2b94ec,#1b75c0);font-weight:700}.sec{background:linear-gradient(#4a5968,#37424f)}.danger{background:linear-gradient(#d56363,#ab4343)}
.m{height:10px;border:1px solid var(--b);border-radius:99px;overflow:hidden;background:#0e1822;margin-top:8px}.m>span{display:block;height:100%;width:0}
.f{font-size:.85rem;color:var(--muted);min-height:1rem;margin-top:5px}.status{margin-top:10px;white-space:pre-wrap;background:#111c28;border:1px solid var(--b);border-radius:8px;padding:10px}
@media(max-width:1100px){.wrap{width:100vw;padding:8px;margin:0}}
</style></head><body><div class='wrap'>
<div class='panel top'><b id='clock'>-- --</b><span id='tz'>UTC</span><div class='sp'></div><label style='margin:0'>Language</label><select id='lang' style='width:180px'><option value='en' selected>English</option><option value='it'>Italiano</option></select><button id='time' style='width:180px'>Time settings</button></div>
<h1 id='title'>First boot configuration</h1><p id='sub' class='sub'>Suggested users: sysadmin, fieldtech, operator.</p><div class='cards' id='cards'></div>
<div class='panel' style='margin-top:10px'><div class='btns'><button id='apply'>Apply configuration</button><button id='backup' class='sec'>Backup recovery</button><button id='reset' class='danger'>Factory reset</button></div><div id='status' class='status'>Ready.</div></div>
</div><script>
const txt={en:{title:'First boot configuration',sub:'Suggested users: sysadmin, fieldtech, operator.',roles:['System admin','Installer','End user'],u:'Username',n:'Full name',p:'Password',perm:'Permissions',apply:'Apply configuration',time:'Time settings',perms:['Full administrator','Network and system time','Read-only']},it:{title:'Configurazione primo avvio',sub:'Utenti suggeriti: sysadmin, fieldtech, operator.',roles:['Amministratore di sistema','Installatore','Utente finale'],u:'Username',n:'Nome esteso',p:'Password',perm:'Permessi',apply:'Applica configurazione',time:'Configura orario',perms:['Amministratore completo','Rete e ora di sistema','Sola visualizzazione']}};
const users=[['admin','sysadmin','System Administrator',0],['installer','fieldtech','Field Installer',1],['viewer','operator','End User Operator',2]];
const score=v=>(v.length>=12)+( /[a-z]/.test(v))+( /[A-Z]/.test(v))+( /\d/.test(v))+( /[^\w]/.test(v));
function style(s){return s<=2?['40%','var(--bad)','Weak']:s<=4?['70%','var(--warn)','Medium']:['100%','var(--ok)','Strong']}
function render(l='en'){const t=txt[l],c=document.getElementById('cards');c.innerHTML='';users.forEach((u,i)=>{const d=document.createElement('div');d.className='card';d.innerHTML=`<h3>${t.roles[i]}</h3><label>${t.u}</label><input id='${u[0]}_u' value='${u[1]}'/><label>${t.n}</label><input id='${u[0]}_n' value='${u[2]}'/><label>${t.p}</label><input type='password' id='${u[0]}_p'/><div class='m'><span id='${u[0]}_m'></span></div><div class='f' id='${u[0]}_f'></div><label>${t.perm}</label><select id='${u[0]}_perm'>${t.perms.map((p,x)=>`<option value='${x}' ${x===u[3]?'selected':''}>${p}</option>`).join('')}</select>`;c.appendChild(d);d.querySelector(`#${u[0]}_p`).oninput=e=>{const [w,col,m]=style(score(e.target.value));d.querySelector(`#${u[0]}_m`).style.cssText=`width:${w};background:${col}`;d.querySelector(`#${u[0]}_f`).textContent=m;};});
document.getElementById('title').textContent=t.title;document.getElementById('sub').textContent=t.sub;document.getElementById('apply').textContent=t.apply;document.getElementById('time').textContent=t.time;}
async function req(path,data=''){const r=await fetch(path,{method:'POST',body:data});return await r.text();}
function payload(){return users.map(u=>`${u[0]}|${gid(u[0]+'_u')}|${gid(u[0]+'_n')}|${gid(u[0]+'_p')}|${gid(u[0]+'_perm')}`).join('\n')}
const gid=id=>document.getElementById(id).value;const set=s=>document.getElementById('status').textContent=s;
async function time(){const t=await (await fetch('/api/time')).text();const [d,h,z]=t.split('|');document.getElementById('clock').textContent=d+' '+h;document.getElementById('tz').textContent=z;}
document.getElementById('lang').onchange=e=>render(e.target.value);document.getElementById('apply').onclick=async()=>set(await req('/api/apply',payload()));document.getElementById('backup').onclick=async()=>set(await req('/api/backup'));document.getElementById('reset').onclick=async()=>set(await req('/api/factory-reset'));
document.getElementById('time').onclick=async()=>{const d=prompt('Date YYYY-MM-DD');const h=prompt('Time HH:MM:SS');const z=prompt('Timezone','UTC');if(d&&h&&z){set(await req('/api/time-settings',`${d}|${h}|${z}`));time();}};
render('en');time();setInterval(time,1000);
</script></body></html>"#;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:22346").expect("cannot bind 22346");
    println!("UI available at http://0.0.0.0:22346");

    for stream in listener.incoming().flatten() {
        thread::spawn(|| handle_connection(stream));
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 65535];
    let bytes = match stream.read(&mut buffer) {
        Ok(0) | Err(_) => return,
        Ok(n) => n,
    };
    let request = String::from_utf8_lossy(&buffer[..bytes]).to_string();
    let mut lines = request.lines();
    let first = lines.next().unwrap_or_default();
    let mut parts = first.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let path = parts.next().unwrap_or("/");
    let body = request.split("\r\n\r\n").nth(1).unwrap_or_default().trim();

    let (status, content_type, response_body) = match (method, path) {
        ("GET", "/") => ("200 OK", "text/html; charset=utf-8", APP_HTML.to_string()),
        ("GET", "/api/time") => (
            "200 OK",
            "text/plain; charset=utf-8",
            format!(
                "{}|{}|{}",
                cmd("date", &["+%Y-%m-%d"]).unwrap_or_else(|| "--/--/----".to_string()),
                cmd("date", &["+%H:%M:%S"]).unwrap_or_else(|| "--:--:--".to_string()),
                cmd("date", &["+%Z"]).unwrap_or_else(|| "UTC".to_string())
            ),
        ),
        ("POST", "/api/apply") => (
            "200 OK",
            "text/plain; charset=utf-8",
            apply_on_host(url_decode(body)),
        ),
        ("POST", "/api/backup") => (
            "200 OK",
            "text/plain; charset=utf-8",
            run_host(
                "sh",
                &[
                    "-c",
                    "echo '[backup] requested' >> /tmp/firstboot-actions.log && uname -a",
                ],
            ),
        ),
        ("POST", "/api/factory-reset") => (
            "200 OK",
            "text/plain; charset=utf-8",
            run_host(
                "sh",
                &[
                    "-c",
                    "echo '[factory-reset] requested' >> /tmp/firstboot-actions.log && date",
                ],
            ),
        ),
        ("POST", "/api/time-settings") => (
            "200 OK",
            "text/plain; charset=utf-8",
            save_time(url_decode(body)),
        ),
        _ => (
            "404 NOT FOUND",
            "text/plain; charset=utf-8",
            "Not Found".to_string(),
        ),
    };

    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response_body.len(),
        response_body
    );
    let _ = stream.write_all(response.as_bytes());
}

fn apply_on_host(payload: String) -> String {
    let escaped = payload.replace('"', "\\\"");
    run_host(
        "sh",
        &[
            "-c",
            &format!("echo \"{escaped}\" >> /tmp/firstboot-user-config.log && echo 'configuration request executed on host'"),
        ],
    )
}

fn save_time(payload: String) -> String {
    let parts: Vec<&str> = payload.split('|').collect();
    if parts.len() != 3 {
        return "Invalid payload for time settings".to_string();
    }
    let datetime = format!("{} {}", parts[0], parts[1]);
    let tz = run_host("timedatectl", &["set-timezone", parts[2]]);
    let dt = run_host("timedatectl", &["set-time", &datetime]);
    format!("timezone:\n{tz}\n\ntime:\n{dt}")
}

fn run_host(program: &str, args: &[&str]) -> String {
    match Command::new(program).args(args).output() {
        Ok(out) => {
            let output = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            if out.status.success() {
                format!("OK: {} {}\n{}", program, args.join(" "), output.trim())
            } else {
                format!("ERROR: {} {}\n{}", program, args.join(" "), output.trim())
            }
        }
        Err(e) => format!("ERROR: {}", e),
    }
}

fn cmd(program: &str, args: &[&str]) -> Option<String> {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn url_decode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hex = &input[i + 1..i + 3];
                if let Ok(v) = u8::from_str_radix(hex, 16) {
                    out.push(v as char);
                    i += 3;
                } else {
                    out.push('%');
                    i += 1;
                }
            }
            b => {
                out.push(b as char);
                i += 1;
            }
        }
    }
    out
}
