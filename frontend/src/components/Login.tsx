import { FormEvent, useState } from 'react'
import { login } from '../lib/api'

export function Login({ onSuccess }: { onSuccess: () => void }) {
  const [password, setPassword] = useState('')
  const [message, setMessage] = useState('')
  const [busy, setBusy] = useState(false)

  async function submit(e: FormEvent) {
    e.preventDefault()
    if (!password) return
    setBusy(true)
    setMessage('')
    try {
      await login(password)
      setPassword('')
      onSuccess()
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error))
    } finally {
      setBusy(false)
    }
  }

  return (
    <div className="login-screen">
      <form className="card login-card" onSubmit={submit}>
        <img src="/oberiz-logo.png" alt="Oberiz" className="login-logo" />
        <h1>Sign in</h1>
        <p>This Oberiz instance is protected with an admin password.</p>
        <label>
          Password
          <input
            type="password"
            autoFocus
            autoComplete="current-password"
            value={password}
            onChange={e => setPassword(e.target.value)}
          />
        </label>
        {message && <div className="error-box">{message}</div>}
        <button type="submit" className="primary-button" disabled={busy || !password}>
          {busy ? 'Signing in…' : 'Sign in'}
        </button>
      </form>
    </div>
  )
}
