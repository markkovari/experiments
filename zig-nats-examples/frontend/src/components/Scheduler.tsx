import { useState, useEffect } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Play, Calendar, History, Clock, Trash2 } from 'lucide-react'
import { Organization } from '../App'
import { useToast } from "@/hooks/use-toast"

import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion"

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

export function Scheduler({ token, org, currentRole }: { token: string, org: Organization | null, currentRole: string }) {
  const [actions, setActions] = useState<Action[]>([])
  const [executions, setExecutions] = useState<Execution[]>([])
  const [actionType, setActionType] = useState('email.send')
  const [cron, setCron] = useState('')
  const { toast } = useToast()
  
  const fetchData = async () => {
    if (!org) return
    try {
      const orgQuery = `?org_id=${org._id.$oid}`
      const [actionsRes, execsRes] = await Promise.all([
        fetch(`/api/actions${orgQuery}`, { headers: { 'Authorization': `Bearer ${token}` } }),
        fetch(`/api/executions${orgQuery}`, { headers: { 'Authorization': `Bearer ${token}` } })
      ])
      if (actionsRes.ok) {
         const fetchedActions = await actionsRes.json();
         setActions(fetchedActions);
         
         if (execsRes.ok) {
           const fetchedExecutions = await execsRes.json();
           setExecutions(fetchedExecutions);
         }
      }
    } catch (e) {
      console.error("Failed to fetch scheduler data", e)
    }
  }

  useEffect(() => {
    setActions([])
    setExecutions([])
    fetchData()
    const interval = setInterval(fetchData, 5000)
    return () => clearInterval(interval)
  }, [token, org])

  const createAction = async (type: 'Manual' | 'Cron') => {
    if (!org) return
    try {
      const response = await fetch(`/api/actions`, {
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
      const response = await fetch(`/api/actions/${id}/trigger`, {
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

  const deleteAction = async (id: string) => {
    try {
      const response = await fetch(`/api/actions/${id}`, {
        method: 'DELETE',
        headers: { 'Authorization': `Bearer ${token}` }
      })
      if (response.ok) {
        toast({ title: "Action deleted" })
        fetchData()
      }
    } catch {
      toast({ variant: "destructive", title: "Failed to delete action" })
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

  const canManageActions = currentRole === 'Owner' || currentRole === 'Admin' || currentRole === 'Editor';
  const canRunActions = canManageActions || currentRole === 'Invoker';

  return (
    <div className="space-y-8">
      {canManageActions && (
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
          )}

          <Card className="shadow-sm border-zinc-200">

        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <History className="w-5 h-5" />
            Recent Executions
          </CardTitle>
          <CardDescription>Track the outcome of scheduled and manual events.</CardDescription>
        </CardHeader>
        <CardContent>
          <Accordion type="single" collapsible className="w-full">
            {executions.map((ex) => (
              <AccordionItem key={ex._id.$oid} value={ex._id.$oid}>
                <AccordionTrigger className="hover:no-underline py-2">
                  <div className="flex items-center justify-between w-full pr-4">
                    <div className="flex items-center gap-4">
                      <span className="font-medium">
                        {actions.find(a => a._id.$oid === ex.action_id.$oid)?.action_type || 'Unknown'}
                      </span>
                      <Badge variant={
                        ex.status === 'Completed' ? 'default' : 
                        ex.status === 'Running' ? 'secondary' : 
                        ex.status === 'Failed' ? 'destructive' : 'outline'
                      }>
                        {ex.status}
                      </Badge>
                    </div>
                    <span className="text-zinc-500 text-xs">
                      {new Date(ex.started_at).toLocaleString()}
                    </span>
                  </div>
                </AccordionTrigger>
                <AccordionContent className="pt-2 pb-4 px-1">
                  <div className="space-y-4 text-xs font-mono bg-zinc-50 p-4 rounded-lg border border-zinc-100">
                    <div>
                      <p className="text-[10px] uppercase font-bold text-zinc-400 mb-1">Payload</p>
                      <pre className="overflow-auto whitespace-pre-wrap break-all">
                        {JSON.stringify(actions.find(a => a._id.$oid === ex.action_id.$oid)?.payload, null, 2)}
                      </pre>
                    </div>
                    {ex.result && (
                      <div>
                        <p className="text-[10px] uppercase font-bold text-zinc-400 mb-1">Result</p>
                        <pre className="text-primary overflow-auto whitespace-pre-wrap break-all">
                          {JSON.stringify(ex.result, null, 2)}
                        </pre>
                      </div>
                    )}
                    {ex.completed_at && (
                      <p className="text-[10px] text-zinc-400">
                        Completed at: {new Date(ex.completed_at).toLocaleString()}
                      </p>
                    )}
                  </div>
                </AccordionContent>
              </AccordionItem>
            ))}
          </Accordion>
          {executions.length === 0 && (
            <div className="text-center py-12 text-zinc-400 border-2 border-dashed rounded-lg">
              No executions yet
            </div>
          )}
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
              <div className="flex gap-2 items-center">
                {action.trigger_type === 'Manual' && canRunActions && (
                  <Button size="sm" variant="ghost" onClick={() => triggerAction(action._id.$oid)}>
                    Run Now
                  </Button>
                )}
                {canManageActions && (
                  <Button size="sm" variant="ghost" onClick={() => deleteAction(action._id.$oid)}>
                    <Trash2 className="w-4 h-4 text-destructive" />
                  </Button>
                )}
              </div>
            </Card>
          ))}
        </div>
      </div>
    </div>
  )
}

