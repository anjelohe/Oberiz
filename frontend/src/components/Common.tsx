import { ReactNode } from 'react'

export function StatCard({ icon, label, value, detail }: { icon: ReactNode; label: string; value: string; detail: string }) {
  return (
    <section className="stat-card card">
      <div className="stat-icon">{icon}</div>
      <div className="stat-copy">
        <span>{label}</span>
        <strong>{value}</strong>
        <small>{detail}</small>
      </div>
      <div className="spark"><i /><i /><i /><i /></div>
    </section>
  )
}

export function Panel({ title, action, children }: { title: string; action?: ReactNode; children: ReactNode }) {
  return (
    <section className="card panel">
      <div className="panel-head">
        <h3>{title}</h3>
        {action}
      </div>
      {children}
    </section>
  )
}

export function Badge({ children, tone = 'green' }: { children: ReactNode; tone?: 'green' | 'blue' | 'yellow' | 'red' }) {
  return <span className={`badge badge-${tone}`}>{children}</span>
}
