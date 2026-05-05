import { useState, useEffect } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Button } from "@/components/ui/button"
import { Plus, Users, ShieldCheck } from 'lucide-react'
import { Organization } from '../App'
import { useToast } from "@/hooks/use-toast"
import { cn } from '@/lib/utils'

interface OrgsProps {
  token: string;
  onSelectOrg: (org: Organization) => void;
  selectedOrg: Organization | null;
}

export function Organizations({ token, onSelectOrg, selectedOrg }: OrgsProps) {
  const [orgs, setOrgs] = useState<Organization[]>([])
  const [newOrgName, setNewOrgName] = useState('')
  const { toast } = useToast()
  
  const fetchOrgs = async () => {
    try {
      const response = await fetch(`/api/orgs`, {
        headers: { 'Authorization': `Bearer ${token}` }
      })
      if (response.ok) {
        const data = await response.json()
        setOrgs(data)
        if (data.length > 0 && !selectedOrg) {
          onSelectOrg(data[0])
        }
      }
    } catch (err) {
      console.error("Failed to fetch orgs", err)
    }
  }

  useEffect(() => {
    fetchOrgs()
  }, [token])

  const createOrg = async () => {
    if (!newOrgName) return
    try {
      const response = await fetch(`/api/orgs`, {
        method: 'POST',
        headers: { 
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({ name: newOrgName }),
      })
      if (response.ok) {
        const data = await response.json()
        setOrgs([...orgs, data])
        onSelectOrg(data)
        setNewOrgName('')
        toast({ title: "Organization created" })
      }
    } catch {
      toast({ variant: "destructive", title: "Failed to create organization" })
    }
  }

  return (
    <div className="space-y-6">
      <Card className="shadow-sm border-zinc-200">
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <Users className="w-5 h-5" />
            Organizations
          </CardTitle>
          <CardDescription>Switch or create work spaces.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex gap-2">
            <Input 
              placeholder="Org Name" 
              value={newOrgName}
              onChange={(e) => setNewOrgName(e.target.value)}
            />
            <Button size="icon" onClick={createOrg} disabled={!newOrgName}>
              <Plus className="w-4 h-4" />
            </Button>
          </div>

          <div className="space-y-2">
            {orgs.map((org) => (
              <button
                key={org._id.$oid}
                onClick={() => onSelectOrg(org)}
                className={cn(
                  "w-full text-left px-4 py-3 rounded-lg border transition-all flex items-center justify-between group",
                  selectedOrg?._id.$oid === org._id.$oid 
                    ? "bg-zinc-900 border-zinc-900 text-white shadow-md" 
                    : "bg-white border-zinc-200 hover:border-zinc-300 text-zinc-700"
                )}
              >
                <span className="font-medium">{org.name}</span>
                <ShieldCheck className={cn(
                  "w-4 h-4 opacity-0 transition-opacity",
                  selectedOrg?._id.$oid === org._id.$oid ? "opacity-100" : "group-hover:opacity-50"
                )} />
              </button>
            ))}
            {orgs.length === 0 && (
              <div className="text-center py-8 border-2 border-dashed rounded-lg text-zinc-400 text-sm">
                No organizations yet
              </div>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
