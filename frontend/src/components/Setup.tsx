import { FormEvent, useState } from 'react'
import { login, setAdminPassword } from '../lib/api'

export function Setup({ onSuccess }: { onSuccess: () => void }) {
  const [password, setPassword] = useState('')
  const [confirm, setConfirm] = useState('')
  const [message, setMessage] = useState('')
  const [busy, setBusy] = useState(false)

  async function submit(e: FormEvent) {
    e.preventDefault()
    if (password.length < 8) { setMessage('Password must be at least 8 characters'); return }
    if (password !== confirm) { setMessage('Passwords do not match'); return }
    setBusy(true)
    setMessage('')
    try {
      await setAdminPassword(null, password)
      await login(password)
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
        <h1>Create an admin password</h1>
        <p>Set a password once to protect this Oberiz instance before using it.</p>
        <label>
          Password
          <input
            type="password"
            autoFocus
            autoComplete="new-password"
            value={password}
            onChange={e => setPassword(e.target.value)}
          />
        </label>
        <label>
          Confirm password
          <input
            type="password"
            autoComplete="new-password"
            value={confirm}
            onChange={e => setConfirm(e.target.value)}
          />
        </label>
        {message && <div className="error-box">{message}</div>}
        <button type="submit" className="primary-button" disabled={busy || !password || !confirm}>
          {busy ? 'Saving…' : 'Continue'}
        </button>
      </form>
    </div>
  )
}
