import { useCallback, useEffect, useState, type FormEvent } from "react"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { DateTimePicker } from "@/components/datetime-picker"
import {
  api,
  ApiError,
  type Appointment,
  type AppointmentsResponse,
  type Pet,
  type PetsResponse,
} from "@/lib/api"

function describe(err: unknown): string {
  if (err instanceof ApiError || err instanceof Error) return err.message
  return String(err)
}

export function OwnerView() {
  const [pets, setPets] = useState<Pet[]>([])
  const [appts, setAppts] = useState<Appointment[]>([])
  const [error, setError] = useState("")

  // add-pet form
  const [petName, setPetName] = useState("")
  const [petSpecies, setPetSpecies] = useState("")

  // search
  const [query, setQuery] = useState("")

  // book form
  const [apptPet, setApptPet] = useState("")
  const [apptWhen, setApptWhen] = useState("")

  const loadPets = useCallback(async (q?: string) => {
    const path = q ? `/pets?q=${encodeURIComponent(q)}` : "/pets"
    const { pets } = await api<PetsResponse>("GET", path)
    setPets(pets)
    if (pets.length && !pets.some((p) => p.id === apptPet)) {
      setApptPet(pets[0].id)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const loadAppts = useCallback(async () => {
    const { appointments } = await api<AppointmentsResponse>(
      "GET",
      "/appointments",
    )
    setAppts(appointments)
  }, [])

  useEffect(() => {
    void loadPets().catch((e) => setError(describe(e)))
    void loadAppts().catch((e) => setError(describe(e)))
  }, [loadPets, loadAppts])

  async function handleAddPet(e: FormEvent) {
    e.preventDefault()
    setError("")
    try {
      await api("POST", "/pets", { name: petName, species: petSpecies })
      setPetName("")
      setPetSpecies("")
      await loadPets(query || undefined)
    } catch (err) {
      setError(describe(err))
    }
  }

  async function handleSearch(e: FormEvent) {
    e.preventDefault()
    setError("")
    try {
      await loadPets(query || undefined)
    } catch (err) {
      setError(describe(err))
    }
  }

  async function handleClear() {
    setQuery("")
    setError("")
    try {
      await loadPets()
    } catch (err) {
      setError(describe(err))
    }
  }

  async function handleBook(e: FormEvent) {
    e.preventDefault()
    setError("")
    try {
      await api("POST", "/appointments", { pet: apptPet, datetime: apptWhen })
      setApptWhen("")
      await loadAppts()
    } catch (err) {
      setError(describe(err))
    }
  }

  async function handleDeletePet(petId: string) {
    setError("")
    try {
      await api("DELETE", `/pets/${encodeURIComponent(petId)}`)
      await loadPets(query || undefined)
      await loadAppts()
    } catch (err) {
      setError(describe(err))
    }
  }

  async function handleCancelAppt(apptId: string) {
    setError("")
    try {
      await api("DELETE", `/appointments/${encodeURIComponent(apptId)}`)
      await loadAppts()
      await loadPets(query || undefined)
    } catch (err) {
      setError(describe(err))
    }
  }

  // a pet can be deleted only if it has no active (non-cancelled) bookings
  function petHasActiveBooking(petId: string): boolean {
    return appts.some((a) => a.pet === petId && a.status !== "cancelled")
  }
  // an appointment can be cancelled only when it's more than 24h away
  function apptCancellable(a: Appointment): boolean {
    const when = Date.parse(a.datetime)
    if (Number.isNaN(when)) return false
    return when - Date.now() > 24 * 3_600_000
  }

  return (
    <div className="space-y-6">
      {error && <p className="text-sm text-destructive">{error}</p>}

      <Card>
        <CardHeader>
          <CardTitle>My pets</CardTitle>
          <CardDescription>Register a new pet and search your pets.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <form onSubmit={handleAddPet} className="flex flex-wrap items-end gap-3">
            <div className="space-y-2">
              <Label htmlFor="pet-name">Name</Label>
              <Input
                id="pet-name"
                required
                value={petName}
                onChange={(e) => setPetName(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="pet-species">Species</Label>
              <Input
                id="pet-species"
                required
                value={petSpecies}
                onChange={(e) => setPetSpecies(e.target.value)}
              />
            </div>
            <Button type="submit">Add pet</Button>
          </form>

          <form onSubmit={handleSearch} className="flex flex-wrap items-end gap-3">
            <div className="flex-1 space-y-2">
              <Label htmlFor="pet-q">Search</Label>
              <Input
                id="pet-q"
                placeholder="search pets…"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
              />
            </div>
            <Button type="submit" variant="secondary">
              Search
            </Button>
            <Button type="button" variant="ghost" onClick={handleClear}>
              Clear
            </Button>
          </form>

          {pets.length === 0 ? (
            <p className="text-sm text-muted-foreground">No pets yet.</p>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Species</TableHead>
                  <TableHead>Notes</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {pets.map((p) => {
                  const blocked = petHasActiveBooking(p.id)
                  return (
                    <TableRow key={p.id}>
                      <TableCell className="font-medium">{p.name}</TableCell>
                      <TableCell>{p.species}</TableCell>
                      <TableCell className="text-muted-foreground">
                        {p.notes ?? "—"}
                      </TableCell>
                      <TableCell className="text-right">
                        <Button
                          variant="destructive"
                          size="sm"
                          disabled={blocked}
                          title={blocked ? "Pet has an active booking — cancel it first" : "Delete pet"}
                          onClick={() => handleDeletePet(p.id)}
                        >
                          Delete
                        </Button>
                      </TableCell>
                    </TableRow>
                  )
                })}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Book an appointment</CardTitle>
          <CardDescription>
            Pick one of your pets, then choose a date and time.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleBook} className="flex flex-wrap items-end gap-3">
            <div className="space-y-2">
              <Label htmlFor="appt-pet">Pet</Label>
              <Select value={apptPet} onValueChange={setApptPet}>
                <SelectTrigger id="appt-pet" className="w-48">
                  <SelectValue placeholder="Select a pet" />
                </SelectTrigger>
                <SelectContent>
                  {pets.map((p) => (
                    <SelectItem key={p.id} value={p.id}>
                      {p.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <DateTimePicker value={apptWhen} onChange={setApptWhen} />
            <Button type="submit" disabled={!apptPet || !apptWhen}>
              Book
            </Button>
          </form>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>My appointments</CardTitle>
        </CardHeader>
        <CardContent>
          {appts.length === 0 ? (
            <p className="text-sm text-muted-foreground">No appointments yet.</p>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>ID</TableHead>
                  <TableHead>Pet</TableHead>
                  <TableHead>When</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {appts.map((a) => {
                  const cancellable = apptCancellable(a)
                  return (
                    <TableRow key={a.id}>
                      <TableCell className="font-mono text-xs">{a.id}</TableCell>
                      <TableCell>{a.pet}</TableCell>
                      <TableCell>{a.datetime}</TableCell>
                      <TableCell>
                        <Badge variant="secondary">{a.status}</Badge>
                      </TableCell>
                      <TableCell className="text-right">
                        <Button
                          variant="destructive"
                          size="sm"
                          disabled={!cancellable}
                          title={cancellable ? "Cancel appointment" : "Too late — within 24h of the appointment"}
                          onClick={() => handleCancelAppt(a.id)}
                        >
                          Cancel
                        </Button>
                      </TableCell>
                    </TableRow>
                  )
                })}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
