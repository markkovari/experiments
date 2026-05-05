import { useState, useEffect } from 'react'
import { Toaster } from "@/components/ui/toaster"
import { useToast } from "@/hooks/use-toast"
import { Auth } from './components/Auth'
import { Organizations } from './components/Organizations'
import { Scheduler } from './components/Scheduler'
import { Members } from './components/Members'
import { Button } from '@/components/ui/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { LogOut, LayoutDashboard } from 'lucide-react'

export interface User {
  id: string;
  email: string;
}

export interface OrgMember {
  user_id: { $oid: string };
  email: string;
  role: string;
}

export interface Organization {
  _id: { $oid: string };
  name: string;
  owner_id: { $oid: string };
  members: OrgMember[];
}

function App() {
  const [token, setToken] = useState<string | null>(localStorage.getItem('token'))
  const [currentOrg, setCurrentOrg] = useState<Organization | null>(null)
  const [currentRole, setCurrentRole] = useState<string>('Viewer')
  const { toast } = useToast()

  useEffect(() => {
    if (token) {
      localStorage.setItem('token', token)
    } else {
      localStorage.removeItem('token')
      setCurrentOrg(null)
    }
  }, [token])

  useEffect(() => {
    if (token && currentOrg) {
      try {
        const payload = JSON.parse(atob(token.split('.')[1]));
        const userId = payload.sub;
        const member = currentOrg.members.find(m => m.user_id.$oid === userId);
        if (member) {
          setCurrentRole(member.role);
        }
      } catch (e) {
        console.error("Failed to parse token", e);
      }
    }
  }, [token, currentOrg])

  const logout = () => {
    setToken(null)
    toast({
      title: "Logged out",
      description: "You have been successfully logged out.",
    })
  }

  if (!token) {
    return (
      <div className="min-h-screen bg-zinc-50 flex items-center justify-center p-4">
        <Auth onLogin={setToken} />
        <Toaster />
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-zinc-50 p-4 md:p-8">
      <div className="max-w-6xl mx-auto space-y-8">
        <header className="flex items-center justify-between bg-white p-4 rounded-xl shadow-sm border border-zinc-200">
          <div className="flex items-center gap-2">
            <LayoutDashboard className="w-6 h-6 text-primary" />
            <h1 className="text-xl font-bold tracking-tight">Event Platform</h1>
          </div>
          <div className="flex items-center gap-4">
            {currentOrg && (
              <span className="text-sm font-medium text-zinc-500 bg-zinc-100 px-3 py-1 rounded-full">
                Org: {currentOrg.name} ({currentRole})
              </span>
            )}
            <Button variant="ghost" size="sm" onClick={logout} className="gap-2">
              <LogOut className="w-4 h-4" />
              Logout
            </Button>
          </div>
        </header>

        <main className="grid grid-cols-1 md:grid-cols-3 gap-8">
          <aside className="md:col-span-1">
            <Organizations 
              token={token} 
              onSelectOrg={setCurrentOrg} 
              selectedOrg={currentOrg} 
            />
          </aside>
          <section className="md:col-span-2">
             {currentOrg ? (
              <Tabs defaultValue="scheduler" className="w-full">
                <TabsList className="mb-4">
                  <TabsTrigger value="scheduler">Actions & Executions</TabsTrigger>
                  <TabsTrigger value="members">Team Members</TabsTrigger>
                </TabsList>
                <TabsContent value="scheduler">
                  <Scheduler token={token} org={currentOrg} currentRole={currentRole} />
                </TabsContent>
                <TabsContent value="members">
                  <Members token={token} org={currentOrg} currentRole={currentRole} />
                </TabsContent>
              </Tabs>
             ) : (
                <Scheduler token={token} org={null} currentRole={currentRole} />
             )}
          </section>
        </main>
      </div>
      <Toaster />
    </div>
  )
}

export default App
