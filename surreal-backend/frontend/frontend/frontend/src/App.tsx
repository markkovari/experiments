import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
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
  cost: number | null
}

interface Doctor {
  id: string
  name: string
}

// API functions
const fetchUsers = async (): Promise<User[]> => {
  const res = await fetch(`${API_URL}/users`)
  if (!res.ok) throw new Error('Failed to fetch users')
  return res.json()
}

const fetchPets = async (): Promise<Pet[]> => {
  const res = await fetch(`${API_URL}/pets`)
  if (!res.ok) throw new Error('Failed to fetch pets')
  return res.json()
}

const fetchChecks = async (): Promise<HealthCheck[]> => {
  const res = await fetch(`${API_URL}/checks`)
  if (!res.ok) throw new Error('Failed to fetch checks')
  return res.json()
}

const fetchDoctors = async (): Promise<Doctor[]> => {
  const res = await fetch(`${API_URL}/doctors`)
  if (!res.ok) throw new Error('Failed to fetch doctors')
  return res.json()
}

function App() {
  const queryClient = useQueryClient()
  const [showUserForm, setShowUserForm] = useState(false)
  const [showPetForm, setShowPetForm] = useState<Record<string, boolean>>({})
  const [showCheckForm, setShowCheckForm] = useState<Record<string, boolean>>({})
  const [showEditCheckForm, setShowEditCheckForm] = useState<Record<string, boolean>>({})

  // Queries
  const { data: users = [], isLoading: usersLoading, error: usersError } = useQuery({
    queryKey: ['users'],
    queryFn: fetchUsers,
  })

  const { data: pets = [], isLoading: petsLoading } = useQuery({
    queryKey: ['pets'],
    queryFn: fetchPets,
  })

  const { data: checks = [], isLoading: checksLoading } = useQuery({
    queryKey: ['checks'],
    queryFn: fetchChecks,
  })

  const { data: doctors = [] } = useQuery({
    queryKey: ['doctors'],
    queryFn: fetchDoctors,
  })

  // Mutations
  const createUserMutation = useMutation({
    mutationFn: async (userData: Omit<User, 'id'>) => {
      const res = await fetch(`${API_URL}/users`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(userData)
      })
      if (!res.ok) throw new Error('Failed to create user')
      return res.json()
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['users'] })
      setShowUserForm(false)
    },
  })

  const deleteUserMutation = useMutation({
    mutationFn: async (userId: string) => {
      const res = await fetch(`${API_URL}/users/${userId}`, { method: 'DELETE' })
      if (!res.ok) throw new Error('Failed to delete user')
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['users'] })
    },
  })

  const createPetMutation = useMutation({
    mutationFn: async (petData: Omit<Pet, 'id'>) => {
      const res = await fetch(`${API_URL}/pets`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(petData)
      })
      if (!res.ok) throw new Error('Failed to create pet')
      return res.json()
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pets'] })
    },
  })

  const deletePetMutation = useMutation({
    mutationFn: async (petId: string) => {
      const res = await fetch(`${API_URL}/pets/${petId}`, { method: 'DELETE' })
      if (!res.ok) throw new Error('Failed to delete pet')
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pets'] })
      queryClient.invalidateQueries({ queryKey: ['checks'] })
    },
  })

  const createCheckMutation = useMutation({
    mutationFn: async (checkData: { pet_id: string; doctor_id: string; scheduled_at: string }) => {
      const res = await fetch(`${API_URL}/checks`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(checkData)
      })
      if (!res.ok) throw new Error('Failed to create check')
      return res.json()
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['checks'] })
    },
  })

  const updateCheckMutation = useMutation({
    mutationFn: async ({ id, data }: { id: string; data: Record<string, unknown> }) => {
      const res = await fetch(`${API_URL}/checks/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data)
      })
      if (!res.ok) throw new Error('Failed to update check')
      return res.json()
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['checks'] })
    },
  })

  const deleteCheckMutation = useMutation({
    mutationFn: async (checkId: string) => {
      const res = await fetch(`${API_URL}/checks/${checkId}`, { method: 'DELETE' })
      if (!res.ok) throw new Error('Failed to delete check')
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['checks'] })
    },
  })

  // Handlers
  const createUser = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const formData = new FormData(e.currentTarget)
    const form = e.currentTarget
    createUserMutation.mutate({
      name: formData.get('name') as string,
      email: formData.get('email') as string,
      phone: formData.get('phone') as string,
      address: formData.get('address') as string,
    }, {
      onSuccess: () => {
        form.reset()
      }
    })
  }

  const deleteUser = async (userId: string) => {
    if (!confirm('Are you sure you want to delete this user? All their pets and health checks will remain but be orphaned.')) {
      return
    }
    deleteUserMutation.mutate(userId)
  }

  const createPet = async (e: React.FormEvent<HTMLFormElement>, ownerId: string) => {
    e.preventDefault()
    const formData = new FormData(e.currentTarget)
    const form = e.currentTarget
    createPetMutation.mutate({
      owner_id: ownerId,
      name: formData.get('petName') as string,
      species: formData.get('species') as string,
      breed: formData.get('breed') as string,
      weight_kg: parseFloat(formData.get('weight') as string),
    }, {
      onSuccess: () => {
        form.reset()
      }
    })
  }

  const deletePet = async (petId: string, petName: string) => {
    if (!confirm(`Are you sure you want to delete ${petName}? All their health checks will also be deleted.`)) {
      return
    }
    deletePetMutation.mutate(petId)
  }

  const createCheck = async (e: React.FormEvent<HTMLFormElement>, petId: string) => {
    e.preventDefault()
    const formData = new FormData(e.currentTarget)
    const form = e.currentTarget
    createCheckMutation.mutate({
      pet_id: petId,
      doctor_id: formData.get('doctorId') as string,
      scheduled_at: new Date(formData.get('scheduledAt') as string).toISOString(),
    }, {
      onSuccess: () => {
        form.reset()
      }
    })
  }

  const deleteCheck = async (checkId: string) => {
    if (!confirm('Are you sure you want to delete this health check?')) {
      return
    }
    deleteCheckMutation.mutate(checkId)
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

    updateCheckMutation.mutate({ id: checkId, data: updateData })
    setShowEditCheckForm({ ...showEditCheckForm, [checkId]: false })
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

  const loading = usersLoading || petsLoading || checksLoading
  const error = usersError

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
            <Button
              onClick={() => {
                queryClient.invalidateQueries({ queryKey: ['users'] })
                queryClient.invalidateQueries({ queryKey: ['pets'] })
                queryClient.invalidateQueries({ queryKey: ['checks'] })
                queryClient.invalidateQueries({ queryKey: ['doctors'] })
              }}
              disabled={loading}
              variant="outline"
            >
              {loading ? 'Loading...' : 'Refresh'}
            </Button>
          </div>
        </div>

        {error && (
          <Card className="mb-6 border-destructive">
            <CardHeader>
              <CardTitle className="text-destructive">Error</CardTitle>
              <CardDescription>{error instanceof Error ? error.message : 'An error occurred'}</CardDescription>
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
                <Button type="submit" disabled={createUserMutation.isPending}>
                  {createUserMutation.isPending ? 'Creating...' : 'Create User'}
                </Button>
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
                            <p>✉️ {user.email}</p>
                            <p>📞 {user.phone}</p>
                            <p>🏠 {user.address}</p>
                            <p className="text-xs font-mono">{user.id}</p>
                          </div>
                        </CardDescription>
                      </div>
                      <Button
                        variant="destructive"
                        size="sm"
                        onClick={() => deleteUser(user.id)}
                        disabled={deleteUserMutation.isPending}
                      >
                        Delete User
                      </Button>
                    </div>
                  </CardHeader>
                  <CardContent className="pt-6">
                    <div className="flex justify-between items-center mb-4">
                      <h3 className="text-lg font-semibold">Pets ({userPets.length})</h3>
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => setShowPetForm({...showPetForm, [user.id]: !showPetForm[user.id]})}
                      >
                        {showPetForm[user.id] ? 'Cancel' : '+ Add Pet'}
                      </Button>
                    </div>

                    {showPetForm[user.id] && (
                      <div className="mb-4 p-4 border rounded-lg bg-muted/50">
                        <form onSubmit={(e) => createPet(e, user.id)} className="space-y-3">
                          <div className="grid grid-cols-3 gap-3">
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
                          <Button type="submit" size="sm" disabled={createPetMutation.isPending}>
                            {createPetMutation.isPending ? 'Adding...' : 'Add Pet'}
                          </Button>
                        </form>
                      </div>
                    )}

                    {userPets.length === 0 ? (
                      <p className="text-muted-foreground text-sm">No pets registered</p>
                    ) : (
                      <div className="space-y-4">
                        {userPets.map(pet => {
                          const petChecks = getChecksByPet(pet.id)
                          return (
                            <div key={pet.id} className="border rounded-lg p-4 bg-card">
                              <div className="flex justify-between items-start mb-2">
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
                                    disabled={deletePetMutation.isPending}
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
                                          {doctors.map(doctor => (
                                            <option key={doctor.id} value={doctor.id}>{doctor.name}</option>
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
                                    <Button type="submit" size="sm" disabled={createCheckMutation.isPending}>
                                      {createCheckMutation.isPending ? 'Scheduling...' : 'Schedule Check'}
                                    </Button>
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
                                            <Button type="submit" size="sm" className="w-full" disabled={updateCheckMutation.isPending}>
                                              {updateCheckMutation.isPending ? 'Updating...' : 'Update Check'}
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
                                                    disabled={deleteCheckMutation.isPending}
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
                                              {check.cost !== null && check.cost !== undefined && (
                                                <p className="text-xs mt-1">💰 Cost: ${check.cost.toFixed(2)}</p>
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
