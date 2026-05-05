import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Button } from "@/components/ui/button"
import { Label } from "@/components/ui/label"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { useToast } from "@/hooks/use-toast"

export function Auth({ onLogin }: { onLogin: (token: string) => void }) {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [loading, setLoading] = useState(false)
  const { toast } = useToast()

  const handleAuth = async (type: 'login' | 'register') => {
    setLoading(true)
    try {
      const response = await fetch(`/api/auth/${type}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password }),
      })

      if (!response.ok) {
        const err = await response.text()
        throw new Error(err || `${type} failed`)
      }

      const data = await response.json()
      
      if (type === 'login') {
        onLogin(data.token)
        toast({
          title: "Welcome back!",
          description: "Successfully logged in.",
        })
      } else {
        toast({
          title: "Account created",
          description: "You can now log in with your credentials.",
        })
      }
    } catch (error: any) {
      toast({
        variant: "destructive",
        title: "Authentication error",
        description: error.message,
      })
    } finally {
      setLoading(false)
    }
  }

  return (
    <Card className="w-[400px] shadow-lg">
      <CardHeader>
        <CardTitle className="text-2xl font-bold">Identity</CardTitle>
        <CardDescription>Join the event-driven revolution.</CardDescription>
      </CardHeader>
      <CardContent>
        <Tabs defaultValue="login" className="w-full">
          <TabsList className="grid w-full grid-cols-2 mb-4">
            <TabsTrigger value="login">Login</TabsTrigger>
            <TabsTrigger value="register">Register</TabsTrigger>
          </TabsList>
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="email">Email</Label>
              <Input 
                id="email" 
                type="email" 
                placeholder="m@example.com" 
                value={email}
                onChange={(e) => setEmail(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="password">Password</Label>
              <Input 
                id="password" 
                type="password" 
                value={password}
                onChange={(e) => setPassword(e.target.value)}
              />
            </div>
          </div>
          <TabsContent value="login" className="mt-4">
            <Button 
              className="w-full" 
              onClick={() => handleAuth('login')}
              disabled={loading}
            >
              {loading ? "Authenticating..." : "Sign In"}
            </Button>
          </TabsContent>
          <TabsContent value="register" className="mt-4">
            <Button 
              className="w-full" 
              variant="outline" 
              onClick={() => handleAuth('register')}
              disabled={loading}
            >
              {loading ? "Creating Account..." : "Create Account"}
            </Button>
          </TabsContent>
        </Tabs>
      </CardContent>
    </Card>
  )
}
