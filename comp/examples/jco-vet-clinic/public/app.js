// Vet-clinic SPA. No framework, no build — plain ES modules + fetch. The token
// lives in localStorage; every guarded call sends it as a Bearer header. The
// view shown is chosen from the principal's roles (returned by /auth/me).

const $ = (id) => document.getElementById(id);
let token = localStorage.getItem("vet_token") || null;
let me = null;

async function api(method, path, body) {
  const headers = {};
  if (body) headers["content-type"] = "application/json";
  if (token) headers["authorization"] = `Bearer ${token}`;
  const res = await fetch(path, { method, headers, body: body ? JSON.stringify(body) : undefined });
  const text = await res.text();
  const data = text ? JSON.parse(text) : null;
  if (!res.ok) throw Object.assign(new Error(data?.error || res.statusText), { status: res.status, data });
  return data;
}

function show(view) {
  for (const v of ["login-view", "owner-view", "doctor-view", "admin-view"]) $(v).classList.add("hidden");
  $(view).classList.remove("hidden");
  $("whoami").classList.toggle("hidden", view === "login-view");
}

async function refreshMe() {
  if (!token) return show("login-view");
  try {
    me = await api("GET", "/auth/me");
  } catch {
    token = null;
    localStorage.removeItem("vet_token");
    return show("login-view");
  }
  $("who").textContent = `${me.subject} · ${me.roles.join(", ")}`;
  if (me.roles.includes("admin")) { show("admin-view"); loadAudit(); }
  else if (me.roles.includes("doctor")) { show("doctor-view"); loadDoctorAppts(); }
  else { show("owner-view"); loadPets(); loadOwnerAppts(); }
}

// ---- auth ----
$("login-form").addEventListener("submit", async (e) => {
  e.preventDefault();
  $("login-error").textContent = "";
  try {
    const tp = await api("POST", "/auth/login", { email: $("email").value, password: $("password").value });
    token = tp.accessToken ?? tp.access_token;
    localStorage.setItem("vet_token", token);
    await refreshMe();
  } catch (err) {
    $("login-error").textContent = `Login failed: ${err.message}`;
  }
});
$("register-form").addEventListener("submit", async (e) => {
  e.preventDefault();
  $("login-error").textContent = "";
  try {
    await api("POST", "/auth/register", { email: $("r-email").value, password: $("r-password").value, role: "pet-owner" });
    const tp = await api("POST", "/auth/login", { email: $("r-email").value, password: $("r-password").value });
    token = tp.accessToken ?? tp.access_token;
    localStorage.setItem("vet_token", token);
    await refreshMe();
  } catch (err) {
    $("login-error").textContent = `Register failed: ${err.message}`;
  }
});
$("logout").addEventListener("click", async () => {
  try { await api("POST", "/auth/logout"); } catch {}
  token = null; me = null;
  localStorage.removeItem("vet_token");
  show("login-view");
});

// ---- owner: pets ----
async function loadPets(q) {
  const { pets } = await api("GET", q ? `/pets?q=${encodeURIComponent(q)}` : "/pets");
  $("pets").innerHTML = pets.map((p) => `<li><b>${p.name}</b> <span class="muted">(${p.species})</span></li>`).join("");
  const sel = $("appt-pet");
  sel.innerHTML = pets.map((p) => `<option value="${p.id}">${p.name}</option>`).join("");
}
$("pet-form").addEventListener("submit", async (e) => {
  e.preventDefault();
  await api("POST", "/pets", { name: $("pet-name").value, species: $("pet-species").value });
  $("pet-name").value = ""; $("pet-species").value = "";
  loadPets();
});
$("pet-search").addEventListener("submit", (e) => { e.preventDefault(); loadPets($("pet-q").value); });
$("pet-clear").addEventListener("click", () => { $("pet-q").value = ""; loadPets(); });

// ---- owner: appointments ----
async function loadOwnerAppts() {
  const { appointments } = await api("GET", "/appointments");
  $("owner-appts").innerHTML = appointments.map((a) => `<li>${a.id} · pet ${a.pet} · ${a.datetime} · <span class="muted">${a.status}</span></li>`).join("");
}
$("appt-form").addEventListener("submit", async (e) => {
  e.preventDefault();
  await api("POST", "/appointments", { pet: $("appt-pet").value, datetime: $("appt-when").value });
  $("appt-when").value = "";
  loadOwnerAppts();
});

// ---- doctor ----
async function loadDoctorAppts() {
  const { appointments } = await api("GET", "/appointments");
  $("doctor-appts").innerHTML = appointments.map((a) => `<li>${a.id} · pet ${a.pet} · ${a.datetime}</li>`).join("") || "<li class='muted'>none assigned</li>";
}
$("note-form").addEventListener("submit", async (e) => {
  e.preventDefault();
  try {
    await api("POST", `/appointments/${encodeURIComponent($("note-appt").value)}/notes`, { text: $("note-text").value });
    $("note-text").value = "";
    alert("note saved");
  } catch (err) { alert(`failed: ${err.message}`); }
});

// ---- admin ----
$("role-form").addEventListener("submit", async (e) => {
  e.preventDefault();
  await api("POST", "/admin/assign-role", { subject: $("role-subject").value, role: $("role-name").value });
  alert("role assigned");
});
async function loadAudit() {
  const { events } = await api("GET", "/admin/audit");
  $("audit").querySelector("tbody").innerHTML = events
    .map((ev) => `<tr><td>${ev.timestamp}</td><td>${ev.event}</td><td>${ev.outcome}</td><td>${ev.subject}</td><td>${ev.detail}</td></tr>`)
    .join("");
}
$("audit-refresh").addEventListener("click", loadAudit);

refreshMe();
