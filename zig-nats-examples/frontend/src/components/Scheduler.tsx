import React, { useState, useEffect } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Button } from "@/components/ui/button"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Badge } from "@/components/ui/badge"
import { Play, Calendar, History, Clock } from 'lucide-react'
import { Organization } from '../App'
import { useToast } from "@/hooks/use-toast"

interface Action {
  _id: { $oid: string };
  action_type: string;
  trigger_type: 'Manual' | 'Cron';
  cron_expression?: string;
  payload: any;
}

interface Execution {
  _id: { $oid: string };
  action_id: { $oid: string };
  status: 'Pending' | 'Running' | 'Completed' | 'Failed';
  started_at: string;
  completed_at?: string;
  result?: any;
}

export function Scheduler({ token, org }: { token: string, org: Organization | null }) {
  const [actions, setOrgs] = useState<Action[]>([])
  const [executions, setExecutions] = useState<Execution[]>([])
  const [actionType, setActionType] = useState('email.send')
  const [cron, setCron] = useState('')
  const { toast } = useToast()

  const fetchData = async () => {
    if (!org) return
    try {
      const [actionsRes, execsRes] = await Promise.all([
        fetch('http://localhost:8080/api/actions', { headers: { 'Authorization': `Bearer ${token}` } }),
        fetch('http://localhost:8080/api/executions', { headers: { 'Authorization': `Bearer ${token}` } })
      ])
      if (actionsRes.ok) setOrgs(await actionsRes.json())
      if (execsRes.ok) setExecutions(await execsRes.json())
    } catch (e) {
      console.error("Failed to fetch scheduler data", e)
    }
  }

  useEffect(() => {
    fetchData()
    const interval = setInterval(fetchData, 5000)
    return () => clearInterval(interval)
  }, [token, org])

  const createAction = async (type: 'Manual' | 'Cron') => {
    if (!org) return
    try {
      const response = await fetch('http://localhost:3002/actions', {
        method: 'POST',
        headers: { 
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({ 
          org_id: org._id.$oid,
          action_type: actionType,
          trigger_type: type,
          cron_expression: type === 'Cron' ? cron : null,
          payload: { message: "Hello from UI" }
        }),
      })
      if (response.ok) {
        toast({ title: "Action created" })
        setCron('')
        fetchData()
      }
    } catch {
      toast({ variant: "destructive", title: "Failed to create action" })
    }
  }

  const triggerAction = async (id: string) => {
    try {
      const response = await fetch(`http://localhost:3002/actions/${id}/trigger`, {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${token}` }
      })
      if (response.ok) {
        toast({ title: "Execution started" })
        fetchData()
      }
    } catch {
      toast({ variant: "destructive", title: "Failed to trigger" })
    }
  }

  if (!org) {
    return (
      <div className="flex flex-col items-center justify-center h-[400px] bg-white rounded-xl border-2 border-dashed border-zinc-200 text-zinc-400">
        <Clock className="w-12 h-12 mb-4 opacity-20" />
        <p>Select an organization to manage actions</p>
      </div>
    )
  }

  return (
    <div className="space-y-8">
      <Card className="shadow-sm border-zinc-200 overflow-hidden">
        <CardHeader className="bg-zinc-50 border-b border-zinc-100">
          <CardTitle className="text-lg flex items-center gap-2">
            <Play className="w-5 h-5 text-primary" />
            Control Panel
          </CardTitle>
          <CardDescription>Define how events are triggered in this organization.</CardDescription>
        </CardHeader>
        <CardContent className="p-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <h3 className="text-sm font-semibold uppercase tracking-wider text-zinc-400">Manual Trigger</h3>
              <div className="flex gap-2">
                <Input 
                  placeholder="Action Type (e.g. sync.data)" 
                  value={actionType}
                  onChange={(e) => setActionType(e.target.value)}
                />
                <Button onClick={() => createAction('Manual')} className="shrink-0">
                  Register
                </Button>
              </div>
            </div>

            <div className="space-y-4">
              <h3 className="text-sm font-semibold uppercase tracking-wider text-zinc-400">Cron Schedule</h3>
              <div className="flex gap-2">
                <Input 
                  placeholder="* * * * *" 
                  value={cron}
                  onChange={(e) => setCron(e.target.value)}
                />
                <Button variant="outline" onClick={() => createAction('Cron')} disabled={!cron} className="shrink-0">
                  <Calendar className="w-4 h-4 mr-2" />
                  Schedule
                </Button>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card className="shadow-sm border-zinc-200">
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <History className="w-5 h-5" />
            Recent Executions
          </CardTitle>
          <CardDescription>Track the outcome of scheduled and manual events.</CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Action</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Started</TableHead>
                <TableHead className="text-right">Result</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {executions.map((ex) => (
                <TableRow key={ex._id.$oid}>
                  <TableCell className="font-medium">
                    {actions.find(a => a._id.$oid === ex.action_id.$oid)?.action_type || 'Unknown'}
                  </TableCell>
                  <TableCell>
                    <Badge variant={
                      ex.status === 'Completed' ? 'default' : 
                      ex.status === 'Running' ? 'secondary' : 
                      ex.status === 'Failed' ? 'destructive' : 'outline'
                    }>
                      {ex.status}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-zinc-500 text-xs">
                    {new Date(ex.started_at).toLocaleString()}
                  </TableCell>
                  <TableCell className="text-right text-xs truncate max-w-[200px] text-zinc-400">
                    {ex.result ? JSON.stringify(ex.result) : '-'}
                  </TableCell>
                </TableRow>
              ))}
              {executions.length === 0 && (
                <TableRow>
                  <TableCell colSpan={4} className="text-center py-12 text-zinc-400">
                    No executions yet
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      <div className="space-y-4">
        <h3 className="text-sm font-semibold uppercase tracking-wider text-zinc-400">Configured Actions</h3>
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          {actions.map((action) => (
            <Card key={action._id.$oid} className="p-4 flex items-center justify-between shadow-none border-zinc-200 hover:border-zinc-300 transition-colors">
              <div>
                <p className="font-bold">{action.action_type}</p>
                <p className="text-xs text-zinc-500 flex items-center gap-1">
                  {action.trigger_type === 'Manual' ? (
                    <> <Play className="w-3 h-3" /> Manual </>
                  ) : (
                    <> <Calendar className="w-3 h-3" /> {action.cron_expression} </>
                  )}
                </p>
              </div>
              {action.trigger_type === 'Manual' && (
                <Button size="sm" variant="ghost" onClick={() => triggerAction(action._id.$oid)}>
                  Run Now
                </Button>
              )}
            </Card>
          ))}
        </div>
      </div>
    </div>
  )
}
