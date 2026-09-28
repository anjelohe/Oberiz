import { useEffect, useState } from 'react'
import { Icon } from '../components/Icon'
import { Badge, Panel, StatCard } from '../components/Common'

type Item={id:number;event_type:string;title:string;detail:string|null;level:string;created_at:string}

export function History(){
  const [rows,setRows]=useState<Item[]>([])
  const [error,setError]=useState('')

  useEffect(()=>{
    fetch('/api/history?limit=200').then(async r=>{
      if(!r.ok) throw new Error(await r.text())
      setRows(await r.json())
    }).catch(e=>setError(e instanceof Error?e.message:String(e)))
  },[])

  return <>
    <div className="page-heading"><div><h1><Icon name="history" size={32}/> Activity History</h1><p>Real events recorded by Oberiz.</p></div></div>
    <div className="stats-grid">
      <StatCard icon={<Icon name="history"/>} label="Events" value={String(rows.length)} detail="Loaded" />
      <StatCard icon={<Icon name="movies"/>} label="Movies" value={String(rows.filter(x=>x.event_type.startsWith('movie.')).length)} detail="Events" />
      <StatCard icon={<Icon name="series"/>} label="Series" value={String(rows.filter(x=>x.event_type.startsWith('series.')).length)} detail="Events" />
      <StatCard icon={<Icon name="indexers"/>} label="Indexers" value={String(rows.filter(x=>x.event_type.startsWith('indexers.')).length)} detail="Events" />
    </div>
    {error&&<div className="error-box">{error}</div>}
    <Panel title="Activity History">
      <div className="data-table">
        {rows.map((row,index)=><div className="data-row" key={row.id}>
          <span className="row-index">{String(index+1).padStart(2,'0')}</span>
          <strong>{row.title}</strong>
          <span>{row.detail || row.event_type}</span>
          <Badge tone={row.level==='error'?'red':'green'}>{row.created_at}</Badge>
        </div>)}
        {!rows.length&&<div className="empty-state">No events yet.</div>}
      </div>
    </Panel>
  </>
}
