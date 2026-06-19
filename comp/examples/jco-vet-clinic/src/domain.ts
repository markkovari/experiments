// Vet-clinic domain logic — pure orchestration over the comp components + the
// shared KV store. The example owns NO business component: pets, appointments
// and visit notes are plain records in the same in-memory bucket the wasm
// components use, and every cross-cutting concern (search, validation) is a
// call into a real comp component.

// The shared KV shim — the SAME module the transpiled components import, so the
// records we write here live in the same store search-index reads from.
import { open } from "./shims/keyvalue.js";
// search:index/index and validate:schema/validator, transpiled to JS.
import { index as search } from "../gen/search/search_index.js";
import { validator as validate } from "../gen/validate/validate.js";

const bucket = open("default");
const enc = (s: string) => new TextEncoder().encode(s);
const dec = (b: Uint8Array) => new TextDecoder().decode(b);

function put(key: string, value: unknown): void {
  bucket.set(key, enc(JSON.stringify(value)));
}
function read<T>(key: string): T | undefined {
  const raw = bucket.get(key);
  return raw ? (JSON.parse(dec(raw)) as T) : undefined;
}
function scan<T>(prefix: string): T[] {
  const { keys } = bucket.listKeys(undefined) as { keys: string[] };
  return keys
    .filter((k) => k.startsWith(prefix))
    .map((k) => read<T>(k))
    .filter((v): v is T => v !== undefined);
}

let counter = 0;
function id(prefix: string): string {
  // deterministic-ish unique id without Math.random (fine for an in-proc demo)
  counter += 1;
  return `${prefix}_${Date.now().toString(36)}${counter.toString(36)}`;
}

// ---- types ---------------------------------------------------------------

export interface Pet {
  id: string;
  name: string;
  species: string;
  owner: string; // owner subject id
  notes: string;
}
export interface Appointment {
  id: string;
  pet: string; // pet id
  owner: string;
  doctor: string; // doctor subject id (or "" = unassigned)
  datetime: string; // ISO-ish string
  status: string;
}
export interface VisitNote {
  id: string;
  appointment: string;
  author: string; // doctor subject
  text: string;
  at: number;
}

// ---- validation rules (declarative, enforced by the validate component) ---

const PET_RULES = [
  { field: "name", kind: "text" as const, required: true, minLen: 1, maxLen: 60, minValue: undefined, maxValue: undefined, oneOf: [] },
  { field: "species", kind: "text" as const, required: true, minLen: 1, maxLen: 40, minValue: undefined, maxValue: undefined, oneOf: [] },
];
const APPT_RULES = [
  { field: "pet", kind: "text" as const, required: true, minLen: 1, maxLen: 80, minValue: undefined, maxValue: undefined, oneOf: [] },
  { field: "datetime", kind: "text" as const, required: true, minLen: 4, maxLen: 40, minValue: undefined, maxValue: undefined, oneOf: [] },
];

export interface FieldError {
  field: string;
  code: string;
  message: string;
}

/** Validate a JSON body against rules via the validate component. */
export function check(body: unknown, which: "pet" | "appointment"): FieldError[] {
  const rules = which === "pet" ? PET_RULES : APPT_RULES;
  return validate.validate(JSON.stringify(body), rules) as FieldError[];
}

// ---- pets ----------------------------------------------------------------

export function createPet(input: { name: string; species: string; owner: string; notes?: string }): Pet {
  const pet: Pet = { id: id("pet"), notes: "", ...input };
  put(`pet_${pet.id}`, pet);
  // index it for full-text search (name + species + notes), tagged by owner.
  search.indexDoc(pet.id, `${pet.name} ${pet.species} ${pet.notes}`, [`owner:${pet.owner}`]);
  return pet;
}

export function listPets(forOwner?: string): Pet[] {
  const pets = scan<Pet>("pet_");
  return forOwner ? pets.filter((p) => p.owner === forOwner) : pets;
}

export function searchPets(q: string, forOwner?: string): Pet[] {
  const tags = forOwner ? [`owner:${forOwner}`] : [];
  const hits = search.query(q, "any", tags, 20) as { id: string; score: number }[];
  return hits.map((h) => read<Pet>(`pet_${h.id}`)).filter((p): p is Pet => p !== undefined);
}

export function getPet(petId: string): Pet | undefined {
  return read<Pet>(`pet_${petId}`);
}

// ---- appointments --------------------------------------------------------

export function createAppointment(input: { pet: string; owner: string; doctor?: string; datetime: string }): Appointment {
  const appt: Appointment = {
    id: id("appt"),
    doctor: "",
    status: "booked",
    ...input,
  };
  put(`appt_${appt.id}`, appt);
  return appt;
}

export function listAppointments(filter: { owner?: string; doctor?: string }): Appointment[] {
  let appts = scan<Appointment>("appt_");
  if (filter.owner) appts = appts.filter((a) => a.owner === filter.owner);
  if (filter.doctor) appts = appts.filter((a) => a.doctor === filter.doctor);
  return appts;
}

export function getAppointment(apptId: string): Appointment | undefined {
  return read<Appointment>(`appt_${apptId}`);
}

/** Assign a doctor to an appointment (claim it). No-op if already that doctor. */
export function assignDoctor(apptId: string, doctor: string): Appointment | undefined {
  const appt = getAppointment(apptId);
  if (!appt) return undefined;
  if (appt.doctor !== doctor) {
    appt.doctor = doctor;
    put(`appt_${apptId}`, appt);
  }
  return appt;
}

/** Active (not cancelled) appointments referencing a pet. */
export function activeAppointmentsForPet(petId: string): Appointment[] {
  return scan<Appointment>("appt_").filter((a) => a.pet === petId && a.status !== "cancelled");
}

export type DeleteResult = { ok: true } | { ok: false; reason: string };

/**
 * Delete a pet — only if it has no active bookings. `owner` scopes the action
 * (an owner may only delete their own pet).
 */
export function deletePet(petId: string, owner: string): DeleteResult {
  const pet = getPet(petId);
  if (!pet) return { ok: false, reason: "not_found" };
  if (pet.owner !== owner) return { ok: false, reason: "forbidden" };
  const active = activeAppointmentsForPet(petId);
  if (active.length > 0) return { ok: false, reason: "has_active_bookings" };
  bucket.delete(`pet_${petId}`);
  search.remove(petId); // keep the index in sync
  return { ok: true };
}

/**
 * Delete (cancel) an appointment — only if it's more than 24h away. `owner`
 * scopes the action. `nowSeconds` is injected so the rule is testable.
 */
export function deleteAppointment(apptId: string, owner: string, nowSeconds: number): DeleteResult {
  const appt = getAppointment(apptId);
  if (!appt) return { ok: false, reason: "not_found" };
  if (appt.owner !== owner) return { ok: false, reason: "forbidden" };
  const when = Date.parse(appt.datetime); // ms, NaN if unparseable
  if (Number.isNaN(when)) return { ok: false, reason: "bad_datetime" };
  const hoursAway = (when - nowSeconds * 1000) / 3_600_000;
  if (hoursAway < 24) return { ok: false, reason: "within_24h" };
  bucket.delete(`appt_${apptId}`);
  return { ok: true };
}

// ---- visit notes ---------------------------------------------------------

export function addNote(input: { appointment: string; author: string; text: string }): VisitNote {
  const note: VisitNote = { id: id("note"), at: Math.floor(Date.now() / 1000), ...input };
  put(`note_${note.id}`, note);
  return note;
}

export function notesFor(appointmentId: string): VisitNote[] {
  return scan<VisitNote>("note_").filter((n) => n.appointment === appointmentId);
}

// ---- audit trail ---------------------------------------------------------
// The composed auth-guard wires audit:log/recorder internally and writes each
// event to the shared KV under `al_{timestamp}_{id}`. It does NOT re-export the
// audit `query` interface, so the admin view reads those keys directly. Returns
// newest-first.

export interface AuditEvent {
  id: string;
  timestamp: number;
  event: string;
  outcome: string;
  tenant: string;
  subject: string;
  detail: string;
}

export function readAuditTrail(max = 100): AuditEvent[] {
  const { keys } = bucket.listKeys(undefined) as { keys: string[] };
  return keys
    .filter((k) => k.startsWith("al_"))
    .sort()
    .reverse()
    .slice(0, max)
    .map((k) => {
      const raw = bucket.get(k);
      if (!raw) return undefined;
      const e = JSON.parse(dec(raw)) as Record<string, unknown>;
      return {
        id: String(e.id ?? ""),
        timestamp: Number(e.timestamp ?? 0),
        event: String(e.event ?? ""),
        outcome: String(e.outcome ?? ""),
        tenant: String(e.tenant ?? ""),
        subject: String(e.subject ?? ""),
        detail: String(e.detail ?? ""),
      };
    })
    .filter((e): e is AuditEvent => e !== undefined);
}
