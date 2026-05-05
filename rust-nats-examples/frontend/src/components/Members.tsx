import { useState, useEffect } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Button } from "@/components/ui/button"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Badge } from "@/components/ui/badge"
import { Users, UserPlus, Trash2 } from 'lucide-react'
import { Organization } from '../App'
import { useToast } from "@/hooks/use-toast"

interface OrgMember {
  user_id: { $oid: string };
  email: string;
  role: string;
}

export function Members({ token, org, currentRole }: { token: string, org: Organization | null, currentRole: string }) {
  const [members, setMembers] = useState<OrgMember[]>([])
  const [inviteEmail, setInviteEmail] = useState('')
  const [inviteRole, setInviteRole] = useState('Viewer')
  const { toast } = useToast()
  
  const fetchMembers = async () => {
    if (!org) return
    try {
      const response = await fetch(`/api/orgs/${org._id.$oid}/members`, {
        headers: { 'Authorization': `Bearer ${token}` }
      })
      if (response.ok) {
        const data = await response.json()
        setMembers(data)
      }
    } catch (e) {
      console.error("Failed to fetch members", e)
    }
  }

  useEffect(() => {
    fetchMembers()
  }, [token, org])

  const inviteUser = async () => {
    if (!org || !inviteEmail) return
    try {
      const response = await fetch(`/api/orgs/${org._id.$oid}/members`, {
        method: 'POST',
        headers: { 
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({ email: inviteEmail, role: inviteRole }),
      })
      if (response.ok) {
        toast({ title: "User invited successfully" })
        setInviteEmail('')
        fetchMembers()
      } else {
        const err = await response.text()
        toast({ variant: "destructive", title: "Failed to invite user", description: err })
      }
    } catch {
      toast({ variant: "destructive", title: "Failed to invite user" })
    }
  }

  const removeUser = async (userId: string) => {
    if (!org) return
    try {
      const response = await fetch(`/api/orgs/${org._id.$oid}/members/${userId}`, {
        method: 'DELETE',
        headers: { 'Authorization': `Bearer ${token}` }
      })
      if (response.ok) {
        toast({ title: "User removed" })
        fetchMembers()
      } else {
        const err = await response.text()
        toast({ variant: "destructive", title: "Failed to remove user", description: err })
      }
    } catch {
      toast({ variant: "destructive", title: "Failed to remove user" })
    }
  }

  const updateRole = async (userId: string, newRole: string) => {
    if (!org) return
    try {
      const response = await fetch(`/api/orgs/${org._id.$oid}/members/${userId}`, {
        method: 'PUT',
        headers: { 
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({ role: newRole }),
      })
      if (response.ok) {
        toast({ title: "Role updated" })
        fetchMembers()
      } else {
        const err = await response.text()
        toast({ variant: "destructive", title: "Failed to update role", description: err })
      }
    } catch {
      toast({ variant: "destructive", title: "Failed to update role" })
    }
  }

  if (!org) {
    return null
  }

  const canManage = currentRole === 'Owner' || currentRole === 'Admin'

  return (
    <div className="space-y-8">
      {canManage && (
        <Card className="shadow-sm border-zinc-200">
          <CardHeader className="bg-zinc-50 border-b border-zinc-100">
            <CardTitle className="text-lg flex items-center gap-2">
              <UserPlus className="w-5 h-5 text-primary" />
              Invite Team Member
            </CardTitle>
            <CardDescription>Add a new user to this organization.</CardDescription>
          </CardHeader>
          <CardContent className="p-6">
            <div className="flex gap-4 items-end">
              <div className="space-y-2 flex-1">
                <Input 
                  placeholder="Email Address" 
                  type="email"
                  value={inviteEmail}
                  onChange={(e) => setInviteEmail(e.target.value)}
                />
              </div>
              <div className="space-y-2 w-48">
                <select 
                  className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
                  value={inviteRole}
                  onChange={(e) => setInviteRole(e.target.value)}
                >
                  <option value="Admin">Admin</option>
                  <option value="Editor">Editor</option>
                  <option value="Invoker">Invoker</option>
                  <option value="Viewer">Viewer</option>
                </select>
              </div>
              <Button onClick={inviteUser} disabled={!inviteEmail}>
                Send Invite
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      <Card className="shadow-sm border-zinc-200">
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <Users className="w-5 h-5" />
            Team Directory
          </CardTitle>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Email</TableHead>
                <TableHead>Role</TableHead>
                {canManage && <TableHead className="text-right">Actions</TableHead>}
              </TableRow>
            </TableHeader>
            <TableBody>
              {members.map((member) => (
                <TableRow key={member.user_id.$oid}>
                  <TableCell className="font-medium">
                    {member.email}
                  </TableCell>
                  <TableCell>
                    {canManage && member.role !== 'Owner' ? (
                      <select 
                        className="h-8 rounded border-zinc-200 text-sm"
                        value={member.role}
                        onChange={(e) => updateRole(member.user_id.$oid, e.target.value)}
                      >
                        <option value="Admin">Admin</option>
                        <option value="Editor">Editor</option>
                        <option value="Invoker">Invoker</option>
                        <option value="Viewer">Viewer</option>
                      </select>
                    ) : (
                      <Badge variant={member.role === 'Owner' ? 'default' : 'secondary'}>
                        {member.role}
                      </Badge>
                    )}
                  </TableCell>
                  {canManage && (
                    <TableCell className="text-right">
                      {member.role !== 'Owner' && (
                        <Button size="sm" variant="ghost" onClick={() => removeUser(member.user_id.$oid)}>
                          <Trash2 className="w-4 h-4 text-destructive" />
                        </Button>
                      )}
                    </TableCell>
                  )}
                </TableRow>
              ))}
              {members.length === 0 && (
                <TableRow>
                  <TableCell colSpan={3} className="text-center py-12 text-zinc-400">
                    No members yet
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  )
}
