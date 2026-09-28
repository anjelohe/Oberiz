# Oberiz — API e integración

## Alcance actual

Oberiz expone una API interna para su interfaz y una **Public API v1** para clientes externos. La API v1 permite registrar y consultar peticiones; no pretende todavía ser una compatibilidad completa con Radarr, Sonarr, Overseerr o Seerr.

## Activación y autenticación

Activa **Public API v1** y genera una clave desde Settings. Todas las llamadas requieren `X-Api-Key`:

```http
X-Api-Key: obrz_tu_clave
```

No se utiliza `Authorization: Bearer` en la implementación actual.

## Endpoints públicos actuales

| Método | Ruta | Descripción |
| --- | --- | --- |
| `GET` | `/api/v1/status` | Estado y versión de Oberiz. |
| `GET` | `/api/v1/quality-profiles` | Perfiles de calidad activos para clientes nativos, con `request_quality` (`standard` o `4k`). |
| `GET` | `/api/v1/requests` | Últimas 250 peticiones. |
| `POST` | `/api/v1/requests` | Crea una petición o reaplica una ampliación de temporadas de un cliente nativo. |
| `GET` | `/api/v1/requests/{id}` | Detalle y estado calculado de una petición. |

## Crear una petición

Para Cinetta, `client_name: "cinetta"` y `client_request_id` identifican una solicitud de forma estable. Un reintento sano devuelve la existente; una serie que llegue de nuevo con `requested_seasons` actualiza esa selección en Oberiz para poder añadir temporadas posteriores sin crear otra serie. Esta operación no ejecuta el ciclo global: procesa únicamente ese medio.

Los estados locales de Cinetta no son órdenes destructivas para Oberiz. No existe un endpoint implícito que borre o cancele torrents al cambiar una tarjeta a pendiente; una futura pausa o cancelación remota deberá ser explícita.

```http
POST /api/v1/requests
X-Api-Key: obrz_tu_clave
Content-Type: application/json
```

Película mínima:

```json
{
  "media_type": "movie",
  "tmdb_id": 19995
}
```

Serie con perfil y modo de monitorización:

```json
{
  "media_type": "series",
  "tmdb_id": 1399,
  "quality_profile_id": 4,
  "client_name": "cinetta",
  "client_request_id": "184",
  "monitored": true,
  "monitor_mode": "all",
  "requested_by": "user_42"
}
```

Campos admitidos:

- `media_type`: `movie` o `series`.
- `tmdb_id`: identificador TMDB obligatorio.
- `quality_profile_id`: opcional; se usa el perfil predeterminado si falta.
- `client_name` y `client_request_id`: identifican una petición en el cliente. Juntos hacen el alta idempotente: un reintento devuelve la misma petición de Oberiz.
- `monitored`: opcional; por defecto `true`.
- `monitor_mode`: opcional para series; por defecto `all`.
- `requested_seasons`: opcional para series; lista de números de temporada que se deben monitorizar de forma exacta.
- `monitor_future_seasons`: opcional para series; mantiene el seguimiento de episodios que se emitan en el futuro sin activar las temporadas pasadas no seleccionadas.
- `requested_by`: texto opcional de auditoría.

Tras crear la petición, Oberiz actualiza metadata y programa un ciclo de automatización en segundo plano.

## Estados de petición

Los estados actuales se calculan a partir de la biblioteca y los trabajos de descarga:

```text
movie:  available | downloading | searching
series: available | downloading | searching | monitoring
```

Una petición sin medio asociado se muestra como `pending`. Estos estados no incluyen aún métricas de progreso, calidad o velocidad; los clientes que las necesiten deben consultar la API interna de Oberiz o esperar a una ampliación de la API v1.

## Routing de descargas

El cliente externo no decide rutas, categorías ni etiquetas. Oberiz resuelve el Quality Profile efectivo y aplica:

- categoría de qBittorrent;
- plantilla de tags;
- perfil de idioma y reglas de aceptación;
- política de upgrades y cutoff.

Para clientes nativos, cada perfil lleva además `request_quality`: `standard` o `4k`. Se configura con el check **Perfil 4K para peticiones API** en Profiles. Cinetta filtra primero por tipo (película o serie) y por esta marca: si queda un perfil lo envía directamente; si quedan varios permite elegir solo entre esos perfiles.

La categoría y los tags se verifican después de añadir el torrent. `qb_hash` es la correlación principal entre Oberiz y qBittorrent.

## Cinetta y compatibilidad

Cinetta usa exclusivamente esta API nativa. Consulta perfiles autenticados, conserva el identificador remoto y deja que Oberiz determine rutas, categorías, tags e importación.

La compatibilidad Radarr/Sonarr permanece independiente y solo se activa mediante `overseerr.compat_enabled`. Sus rutas `/radarr/api/v3/*` y `/sonarr/api/v3/*` existen para clientes como Overseerr; no condicionan la API nativa.

Para Overseerr se configura la misma instancia dos veces:

```text
http://host:2032/radarr  → Movies
http://host:2032/sonarr  → Series
```

La capa expone perfiles, carpeta raíz, idiomas, tags, lookup y alta de películas/series. Sonarr resuelve el identificador TVDB de una serie a TMDB dentro de Oberiz. Esta compatibilidad está validada para el flujo de solicitud de Overseerr, pero no pretende replicar toda la API de Radarr/Sonarr.

Una clave compartida puede utilizarse durante la configuración inicial; en una instalación expuesta o con varios clientes se recomienda regenerarla cuando termine la prueba y, en una evolución posterior, usar claves revocables por integración.

## Diagnóstico local

`GET /api/diagnostics` es un endpoint local de solo lectura para soporte. No requiere la clave pública y no devuelve secretos. Incluye versión, sistema, estado de SQLite, configuración de servicios, indexadores, automatización y errores recientes.

## Backup local

La interfaz de Settings consume estas rutas locales, sin exponerlas por la API pública:

- `GET /api/backups`: lista los snapshots disponibles.
- `POST /api/backups`: crea un snapshot SQLite autocontenido y consistente con WAL.
- `GET /api/backups/{filename}/download`: descarga una copia local.
- `POST /api/backups/upload`: incorpora un archivo SQLite como backup restaurable.
- `DELETE /api/backups/{filename}`: elimina una copia local.
- `POST /api/backups/{filename}/restore`: restaura un snapshot compatible tras confirmación explícita de la interfaz.

Los backups se guardan en la carpeta `backups` del directorio desde el que se inicia Oberiz. El programador opcional crea copias cada 1–720 horas y conserva solo el número de copias configurado. La restauración reemplaza datos de Oberiz —incluidos ajustes e historial— y no toca biblioteca multimedia ni torrents externos.
