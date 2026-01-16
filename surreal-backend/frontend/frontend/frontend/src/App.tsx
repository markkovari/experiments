import { useState, useEffect } from 'react'
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

const API_URL = 'http://localhost:3000'

interface User {
  id: string
  name: string
  email: string
  phone: string
  address: string
}

interface Pet {
  id: string
  owner_id: string
  name: string
  species: string
  breed: string
  weight_kg: number
}

interface HealthCheck {
  id: string
  pet_id: string
  doctor_id: string | null
  scheduled_at: string
  status: 'scheduled' | 'in_progress' | 'completed' | 'cancelled'
  notes: string | null
  diagnosis: string | null
  treatment: string | null
}

interface Doctor {
  id: string
  name: string
}

function App() {
  const [users, setUsers] = useState<User[]>([])
  const [pets, setPets] = useState<Pet[]>([])
  const [checks, setChecks] = useState<HealthCheck[]>([])
  const [doctors, setDoctors] = useState<Doctor[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [showUserForm, setShowUserForm] = useState(false)
  const [showPetForm, setShowPetForm] = useState<Record<string, boolean>>({})
  const [showCheckForm, setShowCheckForm] = useState<Record<string, boolean>>({})
  const [showEditCheckForm, setShowEditCheckForm] = useState<Record<string, boolean>>({})

  const fetchData = async () => {
    setLoading(true)
    setError(null)
    try {
      const [usersRes, petsRes, checksRes, doctorsRes] = await Promise.all([
        fetch(`${API_URL}/users`),
        fetch(`${API_URL}/pets`),
        fetch(`${API_URL}/checks`),
        fetch(`${API_URL}/doctors`)
      ])

      if (!usersRes.ok || !petsRes.ok || !checksRes.ok || !doctorsRes.ok) {
        throw new Error('Failed to fetch data')
      }

      const [usersData, petsData, checksData, doctorsData] = await Promise.all([
        usersRes.json(),
        petsRes.json(),
        checksRes.json(),
        doctorsRes.json()
      ])

      setUsers(usersData)
      setPets(petsData)
      setChecks(checksData)
      setDoctors(doctorsData)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to fetch data')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchData()
  }, [])

  const createUser = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const formData = new FormData(e.currentTarget)
    const userData = {
      name: formData.get('name') as string,
      email: formData.get('email') as string,
      phone: formData.get('phone') as string,
      address: formData.get('address') as string,
    }

    try {
      const res = await fetch(`${API_URL}/users`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(userData)
      })
      if (!res.ok) throw new Error('Failed to create user')
      setShowUserForm(false)
      e.currentTarget.reset()
      fetchData()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create user')
    }
  }

  const deleteUser = async (userId: string) => {
    if (!confirm('Are you sure you want to delete this user? All their pets and health checks will remain but be orphaned.')) {
      return
    }

    try {
      const res = await fetch(`${API_URL}/users/${userId}`, {
        method: 'DELETE'
      })
      if (!res.ok) throw new Error('Failed to delete user')
      fetchData()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to delete user')
    }
  }

  const createPet = async (e: React.FormEvent<HTMLFormElement>, ownerId: string) => {
    e.preventDefault()
    const formData = new FormData(e.currentTarget)
    const petData = {
      owner_id: ownerId,
      name: formData.get('petName') as string,
      species: formData.get('species') as string,
      breed: formData.get('breed') as string,
      weight_kg: parseFloat(formData.get('weight') as string),
    }

    try {
      const res = await fetch(`${API_URL}/pets`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(petData)
      })
      if (!res.ok) throw new Error('Failed to create pet')
      e.currentTarget.reset()
      fetchData()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create pet')
    }
  }

  const deletePet = async (petId: string, petName: string) => {
    if (!confirm(`Are you sure you want to delete ${petName}? All their health checks will also be deleted.`)) {
      return
    }

    try {
      const res = await fetch(`${API_URL}/pets/${petId}`, {
        method: 'DELETE'
      })
      if (!res.ok) throw new Error('Failed to delete pet')
      fetchData()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to delete pet')
    }
  }

  const createCheck = async (e: React.FormEvent<HTMLFormElement>, petId: string) => {
    e.preventDefault()
    const formData = new FormData(e.currentTarget)
    const checkData = {
      pet_id: petId,
      doctor_id: formData.get('doctorId') as string,
      scheduled_at: new Date(formData.get('scheduledAt') as string).toISOString(),
    }

    try {
      const res = await fetch(`${API_URL}/checks`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(checkData)
      })
      if (!res.ok) throw new Error('Failed to create check')
      e.currentTarget.reset()
      fetchData()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create check')
    }
  }

  const deleteCheck = async (checkId: string) => {
    if (!confirm('Are you sure you want to delete this health check?')) {
      return
    }

    try {
      const res = await fetch(`${API_URL}/checks/${checkId}`, {
        method: 'DELETE'
      })
      if (!res.ok) throw new Error('Failed to delete health check')
      fetchData()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to delete health check')
    }
  }

  const updateCheck = async (e: React.FormEvent<HTMLFormElement>, checkId: string) => {
    e.preventDefault()
    const formData = new FormData(e.currentTarget)

    const updateData: Record<string, unknown> = {}

    const scheduledAt = formData.get('scheduledAt') as string
    if (scheduledAt) {
      updateData.scheduled_at = new Date(scheduledAt).toISOString()
    }

    const diagnosis = formData.get('diagnosis') as string
    if (diagnosis) updateData.diagnosis = diagnosis

    const treatment = formData.get('treatment') as string
    if (treatment) updateData.treatment = treatment

    const notes = formData.get('notes') as string
    if (notes) updateData.notes = notes

    const cost = formData.get('cost') as string
    if (cost) updateData.cost = parseFloat(cost)

    try {
      const res = await fetch(`${API_URL}/checks/${checkId}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(updateData)
      })
      if (!res.ok) throw new Error('Failed to update health check')
      setShowEditCheckForm({ ...showEditCheckForm, [checkId]: false })
      fetchData()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to update health check')
    }
  }

  const getPetsByOwner = (ownerId: string) => pets.filter(pet => pet.owner_id === ownerId)
  const getChecksByPet = (petId: string) => checks.filter(check => check.pet_id === petId)

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'scheduled': return 'text-blue-600'
      case 'in_progress': return 'text-yellow-600'
      case 'completed': return 'text-green-600'
      case 'cancelled': return 'text-red-600'
      default: return 'text-gray-600'
    }
  }

  return (
    <div className="min-h-screen bg-background p-8">
      <div className="max-w-7xl mx-auto">
        <div className="flex justify-between items-center mb-8">
          <div>
            <h1 className="text-4xl font-bold">Veterinary Clinic</h1>
            <p className="text-muted-foreground mt-2">
              {users.length} users, {pets.length} pets, {checks.length} checks
            </p>
          </div>
          <div className="flex gap-2">
            <Button onClick={() => setShowUserForm(!showUserForm)}>
              {showUserForm ? 'Hide' : '+ New User'}
            </Button>
            <Button onClick={fetchData} disabled={loading} variant="outline">
              {loading ? 'Loading...' : 'Refresh'}
            </Button>
          </div>
        </div>

        {error && (
          <Card className="mb-6 border-destructive">
            <CardHeader>
              <CardTitle className="text-destructive">Error</CardTitle>
              <CardDescription>{error}</CardDescription>
            </CardHeader>
          </Card>
        )}

        {showUserForm && (
          <Card className="mb-6">
            <CardHeader>
              <CardTitle>Create New User</CardTitle>
            </CardHeader>
            <CardContent>
              <form onSubmit={createUser} className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <Label htmlFor="name">Name</Label>
                    <Input id="name" name="name" required />
                  </div>
                  <div>
                    <Label htmlFor="email">Email</Label>
                    <Input id="email" name="email" type="email" required />
                  </div>
                  <div>
                    <Label htmlFor="phone">Phone</Label>
                    <Input id="phone" name="phone" required />
                  </div>
                  <div>
                    <Label htmlFor="address">Address</Label>
                    <Input id="address" name="address" required />
                  </div>
                </div>
                <Button type="submit">Create User</Button>
              </form>
            </CardContent>
          </Card>
        )}

        <div className="space-y-6">
          {loading ? (
            <p className="text-muted-foreground text-center py-8">Loading data...</p>
          ) : users.length === 0 ? (
            <p className="text-muted-foreground text-center py-8">No users found. Create one above!</p>
          ) : (
            users.map((user) => {
              const userPets = getPetsByOwner(user.id)
              return (
                <Card key={user.id} className="overflow-hidden">
                  <CardHeader className="bg-accent/50">
                    <div className="flex justify-between items-start">
                      <div>
                        <CardTitle className="text-2xl">{user.name}</CardTitle>
                        <CardDescription className="mt-2">
                          <div className="space-y-1">
                            <p>📧 {user.email}</p>
                            <p>📱 {user.phone}</p>
                            <p>🏠 {user.address}</p>
                          </div>
                        </CardDescription>
                      </div>
                      <div className="text-right space-y-2">
                        <div>
                          <p className="text-xs text-muted-foreground">User ID</p>
                          <p className="text-xs font-mono">{user.id}</p>
                        </div>
                        <Button 
                          variant="destructive" 
                          size="sm"
                          onClick={() => deleteUser(user.id)}
                        >
                          Delete User
                        </Button>
                      </div>
                    </div>
                  </CardHeader>
                  
                  <CardContent className="pt-6">
                    <div className="flex justify-between items-center mb-4">
                      <h3 className="font-semibold text-lg">Pets ({userPets.length})</h3>
                      <Button 
                        size="sm" 
                        variant="outline"
                        onClick={() => setShowPetForm({...showPetForm, [user.id]: !showPetForm[user.id]})}
                      >
                        {showPetForm[user.id] ? 'Cancel' : '+ Add Pet'}
                      </Button>
                    </div>

                    {showPetForm[user.id] && (
                      <div className="border rounded-lg p-4 mb-4 bg-accent/30">
                        <form onSubmit={(e) => createPet(e, user.id)} className="space-y-3">
                          <div className="grid grid-cols-2 gap-3">
                            <div>
                              <Label htmlFor={`petName-${user.id}`}>Pet Name</Label>
                              <Input id={`petName-${user.id}`} name="petName" required />
                            </div>
                            <div>
                              <Label htmlFor={`species-${user.id}`}>Species</Label>
                              <Input id={`species-${user.id}`} name="species" required />
                            </div>
                            <div>
                              <Label htmlFor={`breed-${user.id}`}>Breed</Label>
                              <Input id={`breed-${user.id}`} name="breed" required />
                            </div>
                            <div>
                              <Label htmlFor={`weight-${user.id}`}>Weight (kg)</Label>
                              <Input id={`weight-${user.id}`} name="weight" type="number" step="0.1" required />
                            </div>
                          </div>
                          <Button type="submit" size="sm">Add Pet</Button>
                        </form>
                      </div>
                    )}

                    {userPets.length === 0 ? (
                      <p className="text-muted-foreground text-sm">No pets registered</p>
                    ) : (
                      <div className="space-y-4">
                        {userPets.map((pet) => {
                          const petChecks = getChecksByPet(pet.id)
                          return (
                            <div
                              key={pet.id}
                              className="border rounded-lg p-4 space-y-3 hover:bg-accent/30 transition-colors"
                            >
                              <div className="flex justify-between items-start">
                                <div>
                                  <h4 className="font-semibold text-lg">🐾 {pet.name}</h4>
                                  <p className="text-sm text-muted-foreground">
                                    {pet.species} • {pet.breed} • {pet.weight_kg}kg
                                  </p>
                                  <p className="text-xs text-muted-foreground font-mono mt-1">{pet.id}</p>
                                </div>
                                <div className="flex gap-2">
                                  <Button 
                                    size="sm" 
                                    variant="outline"
                                    onClick={() => setShowCheckForm({...showCheckForm, [pet.id]: !showCheckForm[pet.id]})}
                                  >
                                    {showCheckForm[pet.id] ? 'Cancel' : '+ Schedule Check'}
                                  </Button>
                                  <Button 
                                    size="sm" 
                                    variant="destructive"
                                    onClick={() => deletePet(pet.id, pet.name)}
                                  >
                                    Delete
                                  </Button>
                                </div>
                              </div>

                              {showCheckForm[pet.id] && (
                                <div className="border-t pt-3">
                                  <form onSubmit={(e) => createCheck(e, pet.id)} className="space-y-3">
                                    <div className="grid grid-cols-2 gap-3">
                                      <div>
                                        <Label htmlFor={`doctor-${pet.id}`}>Doctor</Label>
                                        <select 
                                          id={`doctor-${pet.id}`}
                                          name="doctorId" 
                                          className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                                          required
                                        >
                                          <option value="">Select a doctor</option>
                                          {doctors.map(doc => (
                                            <option key={doc.id} value={doc.id}>{doc.name}</option>
                                          ))}
                                        </select>
                                      </div>
                                      <div>
                                        <Label htmlFor={`scheduledAt-${pet.id}`}>Scheduled Date & Time</Label>
                                        <Input 
                                          id={`scheduledAt-${pet.id}`}
                                          name="scheduledAt" 
                                          type="datetime-local" 
                                          required 
                                        />
                                      </div>
                                    </div>
                                    <Button type="submit" size="sm">Schedule Check</Button>
                                  </form>
                                </div>
                              )}

                              {petChecks.length > 0 && (
                                <div className="border-t pt-3 mt-3">
                                  <p className="text-sm font-medium mb-2">Health Checks ({petChecks.length})</p>
                                  <div className="space-y-2">
                                    {petChecks.map((check) => (
                                      <div
                                        key={check.id}
                                        className="bg-background rounded border p-3 text-sm"
                                      >
                                        {showEditCheckForm[check.id] ? (
                                          <form onSubmit={(e) => updateCheck(e, check.id)} className="space-y-3">
                                            <div className="flex justify-between items-center mb-2">
                                              <h4 className="font-medium">Edit Health Check</h4>
                                              <Button
                                                type="button"
                                                size="sm"
                                                variant="ghost"
                                                onClick={() => setShowEditCheckForm({ ...showEditCheckForm, [check.id]: false })}
                                              >
                                                Cancel
                                              </Button>
                                            </div>
                                            <div>
                                              <Label htmlFor={`scheduledAt-${check.id}`} className="text-xs">Scheduled At</Label>
                                              <Input
                                                id={`scheduledAt-${check.id}`}
                                                name="scheduledAt"
                                                type="datetime-local"
                                                defaultValue={new Date(check.scheduled_at).toISOString().slice(0, 16)}
                                                className="text-xs h-8"
                                              />
                                            </div>
                                            <div>
                                              <Label htmlFor={`diagnosis-${check.id}`} className="text-xs">Diagnosis</Label>
                                              <Input
                                                id={`diagnosis-${check.id}`}
                                                name="diagnosis"
                                                defaultValue={check.diagnosis || ''}
                                                placeholder="Enter diagnosis"
                                                className="text-xs h-8"
                                              />
                                            </div>
                                            <div>
                                              <Label htmlFor={`treatment-${check.id}`} className="text-xs">Treatment</Label>
                                              <Input
                                                id={`treatment-${check.id}`}
                                                name="treatment"
                                                defaultValue={check.treatment || ''}
                                                placeholder="Enter treatment"
                                                className="text-xs h-8"
                                              />
                                            </div>
                                            <div>
                                              <Label htmlFor={`notes-${check.id}`} className="text-xs">Notes</Label>
                                              <Input
                                                id={`notes-${check.id}`}
                                                name="notes"
                                                defaultValue={check.notes || ''}
                                                placeholder="Enter notes"
                                                className="text-xs h-8"
                                              />
                                            </div>
                                            <div>
                                              <Label htmlFor={`cost-${check.id}`} className="text-xs">Cost</Label>
                                              <Input
                                                id={`cost-${check.id}`}
                                                name="cost"
                                                type="number"
                                                step="0.01"
                                                placeholder="Enter cost"
                                                className="text-xs h-8"
                                              />
                                            </div>
                                            <Button type="submit" size="sm" className="w-full">
                                              Update Check
                                            </Button>
                                          </form>
                                        ) : (
                                          <div className="flex justify-between items-start mb-1">
                                            <div className="flex-1">
                                              <div className="flex justify-between items-center mb-1">
                                                <span className={`font-semibold ${getStatusColor(check.status)}`}>
                                                  {check.status.replace('_', ' ').toUpperCase()}
                                                </span>
                                                <div className="flex gap-2 items-center">
                                                  <span className="text-xs text-muted-foreground">
                                                    {new Date(check.scheduled_at).toLocaleString()}
                                                  </span>
                                                  <Button
                                                    size="sm"
                                                    variant="ghost"
                                                    className="h-6 px-2 text-xs"
                                                    onClick={() => setShowEditCheckForm({ ...showEditCheckForm, [check.id]: true })}
                                                  >
                                                    Edit
                                                  </Button>
                                                  <Button
                                                    size="sm"
                                                    variant="ghost"
                                                    className="h-6 px-2 text-xs text-destructive hover:text-destructive"
                                                    onClick={() => deleteCheck(check.id)}
                                                  >
                                                    Delete
                                                  </Button>
                                                </div>
                                              </div>
                                              {check.notes && (
                                                <p className="text-muted-foreground text-xs mt-1">📝 {check.notes}</p>
                                              )}
                                              {check.diagnosis && (
                                                <p className="text-xs mt-1">🩺 Diagnosis: {check.diagnosis}</p>
                                              )}
                                              {check.treatment && (
                                                <p className="text-xs mt-1">💊 Treatment: {check.treatment}</p>
                                              )}
                                              <p className="text-xs text-muted-foreground font-mono mt-1">
                                                Check ID: {check.id}
                                              </p>
                                            </div>
                                          </div>
                                        )}
                                      </div>
                                    ))}
                                  </div>
                                </div>
                              )}
                            </div>
                          )
                        })}
                      </div>
                    )}
                  </CardContent>
                </Card>
              )
            })
          )}
        </div>
      </div>
    </div>
  )
}

export default App
