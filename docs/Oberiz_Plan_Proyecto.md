# Oberiz — Plan de producto

**Estado actual:** 0.9.0. El núcleo de automatización está implementado; la prioridad es estabilizarlo y validar integraciones reales antes de ampliar alcance.

## Principios

1. Automatización fiable antes que nuevas pantallas o clientes torrent.
2. API nativa primero; compatibilidad de terceros como capa aislada.
3. No realizar acciones destructivas por cambios de estado externos implícitos.
4. Cada nueva capacidad debe llegar con pruebas y diagnóstico suficiente.
5. Mantener instalación y operación sencillas para un servidor autoalojado.

## 0.9.0 — Cierre operativo

### Hecho

- RSS/Atom incremental y deduplicado por indexador.
- Matching, scoring, upgrades y grab mediante el flujo existente.
- Public API v1 para Cinetta: perfiles, idempotencia y temporadas solicitadas.
- Compatibilidad opcional para Overseerr mediante Radarr/Sonarr v3.
- Dashboard, biblioteca, Downloads, Calendar, importer e historial.
- Backup y restore de SQLite consistentes con WAL, con validación de esquema antes de restaurar.

### Falta validar

- Feeds RSS reales por tracker.
- E2E de una película y una serie con descarga, importación, rescan y upgrade.
- Recuperación tras error de TMDB, qBittorrent, tracker, filesystem y falta de espacio.
- Contratos de Cinetta y Overseerr con pruebas automatizadas.

## 0.9.1 — Observabilidad y notificaciones

- Exportación de diagnóstico sin secretos.
- Eventos y notificaciones configurables: grab, importación, error, upgrade y seed policy.
- Historial más filtrable y correlación visible entre solicitud, release, job, hash e importación.

## 0.9.2 — Perfiles y formatos

- Custom Formats reutilizables y más expresivos.
- Explicación visual de por qué una release fue aceptada, rechazada o superada.
- Reglas por indexador y perfil sin duplicar configuración.

## 0.9.3 — Resiliencia y seguridad

- Claves API separadas, revocables y con propósito por integración.
- Manejo explícito de timeouts, reintentos y estados de error recuperables.

## 0.9.4 — Distribución

- Instalación guiada y Docker pulido.
- Validación de rutas y permisos durante la configuración inicial.
- Guía NAS y actualización segura.

## Camino a 1.0

Oberiz llegará a 1.0 cuando el flujo cotidiano de películas y series sea fiable sin supervisión manual: alta, matching, grab, descarga, importación, upgrades, seed y diagnóstico.

No se priorizan clientes torrent adicionales, Plex/Jellyfin ni aplicaciones móviles antes de ese cierre.

La cronología y las fases anteriores a 0.9.0 se conservan dentro de [`Oberiz_Memoria_Actualizada.md`](Oberiz_Memoria_Actualizada.md), separadas del estado vigente.
