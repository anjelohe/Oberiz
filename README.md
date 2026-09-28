# Oberiz

Servidor autoalojado de automatización multimedia para películas y series. Oberiz centraliza metadata, indexadores, perfiles de calidad, búsquedas, qBittorrent, importación y biblioteca real en una sola aplicación.

**Versión actual: 0.9.0 — Automatización continua mediante RSS.**

## Estado actual

Oberiz ya cubre el flujo completo:

```text
Añadir contenido desde TMDB
→ perfil de calidad e idioma
→ búsqueda en indexadores Cardigann
→ scoring y selección de release
→ qBittorrent: categoría, etiquetas y hash verificados
→ descarga e importación
→ renombrado, biblioteca, rescan, upgrades e historial
```

Para series maneja temporadas, episodios, monitorización y perfiles heredados. La automatización prioriza una serie completa cuando procede; si no es válida, prueba un pack de temporada y finalmente episodios individuales. Los grabs equivalentes activos se evitan mediante el estado de los trabajos y el infohash de los magnets.

## Funcionalidades incluidas

- Películas y series con metadata de TMDB.
- Library real: archivos indexados, disponibilidad, rescan y upgrades por puntuación.
- Quality Profiles y Language Profiles; perfiles predeterminados por tipo de medio.
- Indexadores Cardigann, catálogo upstream y definiciones privadas locales.
- qBittorrent: conexión, categorías reales, tags por perfil, progreso y operación de torrents.
- Automation periódica, importación, hardlinks/copia/movimiento, reseed y políticas de seed.
- RSS por indexador: sincronización incremental, deduplicación persistente, matching con Wanted, scoring y grab automático trazable.
- Calendar, History, Dashboard con datos reales y Public API v1 para Cinetta.
- Compatibilidad opcional Radarr/Sonarr para clientes como Overseerr, separada de la API nativa.
- Diagnóstico local sin secretos y pruebas automáticas del motor de selección.

## Arquitectura

| Parte | Tecnología | Ubicación |
| --- | --- | --- |
| Backend | Rust, Axum, Tokio, SQLx y SQLite | `backend/` |
| Frontend | React, TypeScript y Vite | `frontend/` |
| Indexadores | Definiciones Cardigann | `config/indexers/` |
| Documentación | Estado, API, roadmap e historial | `docs/` |

El backend escucha en `http://localhost:2032`. El frontend de desarrollo escucha en `http://localhost:5173` y redirige `/api` al backend.

## Arranque local

En una terminal:

```powershell
cd backend
cargo run
```

En otra:

```powershell
cd frontend
npm install
npm run dev
```

Después abre `http://localhost:5173`.

## Configuración inicial

1. Abre **Settings** y añade la credencial de TMDB.
2. Configura qBittorrent y pulsa **Test Connection**.
3. Define las rutas de películas, series y descargas.
4. Sincroniza o añade indexadores y configura perfiles de calidad.
5. Añade una película o una serie y usa Automation cuando la configuración esté verificada.

Las definiciones privadas van en `config/indexers/custom/`. Las bases SQLite, secretos, definiciones privadas y artefactos de compilación están excluidos del control de versiones.

## API y diagnóstico

La API interna vive bajo `/api`. La API pública v1 se habilita desde Settings y usa la cabecera `X-Api-Key`:

```text
GET  /api/v1/status
GET  /api/v1/quality-profiles
GET  /api/v1/requests
POST /api/v1/requests
GET  /api/v1/requests/{id}
```

Cinetta usa la API v1 nativa: consulta perfiles autenticados y enlaza cada petición con un identificador de cliente idempotente. Cada perfil declara explícitamente si recibe peticiones estándar o 4K, de modo que Cinetta solo ofrece destinos de la calidad solicitada. La compatibilidad para Overseerr se activa aparte y expone `/radarr/api/v3/*` y `/sonarr/api/v3/*`; no sustituye ni condiciona la integración nativa.

`GET /api/diagnostics` aporta versión, sistema, estado de SQLite, configuración de servicios, indexadores, automatización y errores recientes sin exponer claves ni contraseñas. RSS expone `GET /api/rss/status` y puede sincronizarse manualmente con `POST /api/rss/run`.

Desde **Settings → Backup & Restore** se pueden crear y restaurar snapshots consistentes de SQLite. Se guardan en `backend/backups` cuando Oberiz se inicia desde `backend`; incluyen los cambios que estén en WAL. Restaurar revierte los datos de Oberiz, pero no modifica archivos multimedia ni torrents de qBittorrent.

## Verificación

```powershell
cd backend
cargo test

cd ..\frontend
npm run build
```

La suite cubre el parser/scoring de releases y la decisión `Complete Series → Season Pack → Episode`. El procedimiento manual de extremo a extremo está documentado en `docs/Oberiz_Memoria_Actualizada.md`.

## Documentación

La documentación se conserva exclusivamente en [`docs/README.md`](docs/README.md): estado técnico, integración API, importación, roadmap, RSS 0.9.0 e historial de versiones. La raíz se mantiene deliberadamente limitada a este README, licencia, configuración y código.
