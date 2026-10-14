import { ReactNode } from 'react'
import { t, useTranslation } from '../lib/i18n'

export function StatCard({ icon, label, value, detail }: { icon: ReactNode; label: string; value: string; detail: string }) {
  useTranslation()
  return (
    <section className="stat-card card">
      <div className="stat-icon">{icon}</div>
      <div className="stat-copy">
        <span>{t(label)}</span>
        <strong>{value}</strong>
        <small>{t(detail)}</small>
      </div>
      <div className="spark"><i /><i /><i /><i /></div>
    </section>
  )
}

export function Panel({ title, action, children }: { title: string; action?: ReactNode; children: ReactNode }) {
  useTranslation()
  return (
    <section className="card panel">
      <div className="panel-head">
        <h3>{t(title)}</h3>
        {action}
      </div>
      {children}
    </section>
  )
}

export function Badge({ children, tone = 'green' }: { children: ReactNode; tone?: 'green' | 'blue' | 'yellow' | 'red' }) {
  return <span className={`badge badge-${tone}`}>{children}</span>
}
