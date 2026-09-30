# Oberiz — Memoria actualizada del proyecto

> Estado del proyecto, decisiones tomadas, trabajo realizado y roadmap previsto.

**Fecha de esta memoria:** 28 de septiembre de 2026  
**Estado actual:** Oberiz v1.0.5 es la versión pública vigente. Incluye biblioteca real, automatización de películas y series, RSS incremental, importación, Calendar, API pública v1, diagnóstico, backup/restore SQLite, perfiles refinados y acceso de administrador protegido.

---

## v1.0.5 — Recuperación del icono de bandeja (2026-09-30)

- El acceso directo instalado relanza el ayudante de bandeja junto con el servicio, restaurando el icono después de usar **Close Oberiz**.

## v1.0.4 — Arranque desde el acceso directo de Windows (2026-09-30)

- El acceso directo instalado inicia el servicio si estaba detenido, espera a que responda `127.0.0.1:2032` y abre entonces el navegador.

## v1.0.3 — Corrección del menú de bandeja de Windows (2026-09-30)

- El icono de bandeja interpreta correctamente los clics de Windows con `NOTIFYICON_VERSION_4`: clic izquierdo abre Oberiz y clic derecho muestra el menú contextual.

## v1.0.2 — Menú de bandeja de Windows (2026-09-30)

- La edición instalada ofrece **Close Oberiz** desde el icono de bandeja: detiene el servicio de Windows y cierra el icono.
- La edición portable mantiene **Exit Oberiz**, que termina su proceso integrado; el clic izquierdo sigue abriendo el navegador.
- Las actualizaciones de Windows reutilizan el servicio existente, lo apuntan al ejecutable recién instalado y lo detienen antes de sustituir archivos.

## v1.0.1 — Búsqueda y selección de releases (2026-09-29)

### Búsqueda e indexadores

- La búsqueda Cardigann conserva el intento inicial con el título de metadata y aplica alternativas solo cuando no obtiene resultados: sin el año final y, después, mediante título original. En series se preserva el identificador de temporada en esas alternativas.
- La capa de compatibilidad Cardigann expone a las definiciones las variables de búsqueda disponibles (`TMDBID`, IMDb completo y corto, año, temporada y episodio). Los identificadores no disponibles permanecen vacíos en lugar de inventarse.
- TVDB es una integración opcional para series: con una clave configurada en Settings, Oberiz solicita un token temporal y resuelve un ID de serie antes de consultar definiciones que usan `TVDBID`. Si TVDB no está configurado o no responde, la búsqueda continúa sin ese dato.
- El lector HTML de resultados ignora cabeceras y filas no publicables cuando un selector amplio coincide con tablas de filtros o con avisos de “sin resultados”. La ausencia normal de releases es diagnóstico de nivel `debug`; los fallos reales de autenticación, red o descarga continúan visibles.
- La prioridad de un indexador se consulta en orden ascendente y se usa para desempatar releases ya aceptadas con la misma puntuación de calidad. Cada perfil puede activar **Prioritize indexer**: en ese caso, tras aceptar una release, la prioridad del indexador prevalece sobre la puntuación de calidad. La tabla de indexadores muestra ese valor; `100` es el valor predeterminado y un número menor es preferente.

### Perfiles de calidad y series

- Se puede clonar un perfil de calidad para crear variantes sin reconfigurar reglas, rutas ni etiquetas manualmente.
- Las búsquedas dirigidas a una temporada descartan releases de otra temporada explícita. Se reconocen las notaciones habituales (`S03E07`, `3x07`, `Season` y `Temporada`).
- El perfil puede preferir packs de temporada/serie completa, incluyendo publicaciones marcadas como `PACK`.
- Para series existen dos límites máximos de tamaño: uno para capítulos individuales y otro específico para temporadas o packs completos. Un release con `S03E07`, `3x07`, `Episode` o `Episodio` usa el límite de capítulo; `S03`, `Temporada 3`, `PACK` o `Complete` sin episodio explícito usa el límite de pack. Si el límite de pack no se configura, se conserva el máximo general como respaldo.
- Los términos de rechazo se comparan como tokens de release. Por ejemplo, bloquear `CAM` sigue bloqueando `CAM.1080p`, sin rechazar títulos que contengan esa secuencia dentro de una palabra.

Los YAML concretos de trackers privados, sus URL, credenciales y registros operativos se mantienen fuera del repositorio. Las notas privadas se guardan localmente y no forman parte de Git.

---

## Base pública v1.0.0 (2026-09-28)

- La numeración pública se reinicia en **v1.0.0**; las publicaciones de prueba anteriores se retiran y esta versión pasa a ser la única referencia para Docker, Linux y Windows.
- La autenticación administrativa usa Argon2id, sesiones opacas revocables almacenadas como hashes, cierre de sesión efectivo y revocación global al cambiar la contraseña.
- El acceso aplica espera progresiva tras intentos fallidos. Para despliegues HTTPS existe `OBERIZ_COOKIE_SECURE=true`, que marca la cookie de sesión como exclusiva de HTTPS.
- Las conexiones a TMDB, qBittorrent, API pública y las rutas de biblioteca no se distribuyen: cada instalación configura sus propias credenciales y rutas.

## Actualización Cinetta (2026-09-28)

- La Public API acepta ampliaciones de una serie de Cinetta usando el mismo `client_name` + `client_request_id`: si llega una nueva selección de `requested_seasons`, sustituye solo el registro remoto de petición y reaplica la monitorización exacta sobre la serie existente. Así T2/T3 se añaden sin crear otra serie ni lanzar el ciclo global de automatización.
- Las peticiones ordinarias siguen siendo idempotentes: un reintento sano devuelve la solicitud ya creada. Si el medio fue eliminado manualmente, se descarta el registro huérfano y se puede recrear.
- Filosofía de seguridad compartida: que Cinetta marque una solicitud como `pending` es una decisión de gestión local; no debe borrar, pausar ni cancelar torrents u órdenes que ya estén en Oberiz. Una futura API explícita de pausa/cancelación deberá ser una acción separada y confirmada.

## Estado operativo y compatibilidad validada (2026-09-28)

- El motor 0.9.0 cubre Movies, Series, perfiles, indexadores, parser/scoring, qBittorrent, importer, biblioteca, RSS, Calendar, Dashboard, History y diagnóstico.
- Cinetta opera contra la Public API v1 nativa; no necesita emular Radarr o Sonarr.
- Overseerr puede usar la compatibilidad opcional de Oberiz con `http://host:2032/radarr` para películas y `http://host:2032/sonarr` para series. La capa incluye lookup, perfiles, idiomas, carpetas raíz, tags, alta de películas y conversión TVDB → TMDB para series.
- La categoría de qBittorrent pertenece al Quality Profile y puede quedarse vacía de forma intencionada; en ese caso el torrent llega sin categorizar. Las etiquetas siguen conservando el tracker/origen.
- La disponibilidad anunciada por un indexador no sustituye los datos de qBittorrent: un magnet puede anunciar seeds y quedarse sin metadatos si no hay peers o trackers accesibles.
- Backup & Restore crea snapshots autocontenidos mediante `VACUUM INTO`, seguros con WAL, y restaura solo backups cuyo esquema coincide con la base activa. La operación revierte los datos de Oberiz, no los archivos multimedia ni qBittorrent.
- Quedan pendientes validación RSS con feeds reales, pruebas de integración Cinetta/Overseerr, E2E reproducible y recuperación ante fallos externos.

## Cómo leer esta memoria

Este documento conserva **todo el recorrido técnico** del proyecto. Las secciones iniciales y las fases antiguas describen decisiones tomadas en su momento; no deben interpretarse como el estado vigente si contradicen el resumen anterior. El estado actual es v1.0.0 y las secciones de evolución consolidada al final recogen los hitos por versión.

---

## 1. Resumen ejecutivo

Oberiz es un proyecto open source orientado a unificar en una sola aplicación funciones equivalentes a las que actualmente se reparten entre herramientas como Prowlarr, Radarr, Sonarr y un cliente torrent externo, inicialmente qBittorrent.

La idea no es copiar internamente esas aplicaciones, sino desarrollar una arquitectura propia que integre películas, series, indexadores, búsquedas, monitorización, scoring de releases, descargas, importación, organización de biblioteca, historial, automatización y una única interfaz web.

Stack inicial:

```text
Frontend: React + TypeScript
Backend: Rust
Framework HTTP: Axum
Runtime async: Tokio
Base de datos: SQLite
Cliente HTTP: Reqwest
Acceso SQL: SQLx
Metadata inicial: TMDB
Cliente torrent inicial: qBittorrent por Web API
Puerto inicial de Oberiz: 2032
```

---

## 2. Nombre del proyecto

Nombre actual: **Oberiz**

Motivos:

- corto;
- fácil de recordar;
- fácil de pronunciar;
- sonoridad tecnológica;
- inspiración indirecta astronómica;
- no limita el proyecto a películas, series o torrents;
- adecuado para GitHub, Docker, interfaz web y branding.

Antes de un lanzamiento público definitivo será conveniente comprobar disponibilidad en GitHub, dominios, posibles marcas registradas y otros proyectos de software con nombre idéntico o parecido.

---

## 3. Identidad visual

Se trabajó primero sobre Atlariz y posteriormente se decidió continuar con Oberiz.

Dirección visual acordada:

- moderna;
- tecnológica;
- minimalista;
- futurista sin exceso de elementos;
- tipografía/logo original;
- evitar depender de fuentes comerciales;
- fondo transparente;
- cuerpo principal oscuro;
- acento en verde eléctrico / verde neón.

El logo definitivo de Oberiz todavía no está cerrado, pero la línea visual preferida es:

```text
Oscuro / azul casi negro
+
verde eléctrico
+
formas geométricas suaves
+
iconografía sutil de flujo, nodos o media
```

---

## 4. Filosofía técnica

Oberiz debe ser:

- open source;
- gratuito;
- sin funciones bloqueadas por pago;
- extensible;
- fácil de instalar;
- fácil de respaldar;
- orientado a servidores domésticos, NAS y Docker;
- modular internamente;
- independiente del código fuente de Prowlarr/Radarr/Sonarr.

No se pretende copiar código directamente de esos proyectos. La compatibilidad se basará en APIs públicas, protocolos públicos, formatos documentados, conceptos de interoperabilidad y definiciones externas.

---

## 5. Licencias revisadas

### Prowlarr

Prowlarr usa GPL-3.0.

Decisión: **no reutilizar código interno de Prowlarr directamente**. Oberiz implementará desde cero las partes que necesite.

### Jackett

Jackett usa GPL-2.0.

Parte del ecosistema de definiciones de indexadores deriva o se sincroniza con Jackett.

### Prowlarr/Indexers

Prowlarr mantiene las definiciones de indexadores en un repositorio separado.

Conclusión práctica:

- no incluir directamente todo `Prowlarr/Indexers` dentro del repositorio de Oberiz;
- no asumir que todo el repositorio puede relicenciarse;
- permitir que Oberiz pueda descargar/sincronizar definiciones desde upstream;
- mantenerlas separadas del código propio.

### Licencia de Oberiz

Todavía no se ha cerrado definitivamente.

Opciones consideradas:

- MIT
- Apache-2.0
- GPL-3.0
- AGPL-3.0

Para un servidor web open source, **AGPL-3.0** sigue siendo una candidata fuerte porque obliga a compartir modificaciones incluso cuando se ofrece el software como servicio por red.

La licencia definitiva debe fijarse antes del lanzamiento público estable.

---

## 6. Indexadores y Cardigann

Oberiz tendrá un **motor propio de indexadores**.

La intención es que sea compatible, en la medida de lo posible, con el formato YAML usado por Prowlarr/Cardigann.

Cardigann permite describir un tracker/indexador mediante YAML:

- login;
- cookies;
- rutas;
- formularios;
- categorías;
- búsquedas;
- selectores HTML;
- título;
- tamaño;
- seeders;
- leechers;
- torrent;
- magnet;
- fecha;
- etc.

Oberiz implementará su propio parser/intérprete.

No se reutilizará el motor original de Cardigann.

---

## 7. Indexadores custom privados

Existen tres definiciones custom privadas ya creadas para trackers concretos.

Decisión cerrada:

**No se publicarán en GitHub.**

Se cargarán desde una carpeta local:

```text
/config/indexers/custom/
```

En Windows, mientras el backend se ejecuta desde `backend/`, la ruta configurada actualmente es:

```text
..\config\indexers\custom
```

La idea es que Oberiz detecte y cargue automáticamente los `.yml` presentes ahí.

Motivo:

- evitar problemas con trackers privados;
- evitar publicar automatizaciones no deseadas por sus administradores;
- separar el motor libre de las definiciones privadas;
- permitir que cada usuario aporte sus propios customs.

---

## 8. Arquitectura prevista de indexadores

```text
Indexadores
├── Built-in
│   ├── Generic Torznab
│   ├── Generic Newznab
│   └── integraciones públicas propias
│
├── Upstream
│   └── definiciones descargadas/sincronizadas desde fuentes externas
│
└── Custom
    └── /config/indexers/custom/*.yml
```

---

## 9. TMDB

TMDB será la fuente inicial de metadata de películas.

Se usa para:

- búsqueda;
- título;
- título original;
- año;
- sinopsis;
- poster;
- backdrop;
- valoración;
- futuras colecciones/sagas;
- géneros;
- IDs externos;
- fechas de estreno.

Cada instalación tendrá su propia credencial TMDB.

### Credenciales TMDB

Se comprobó que TMDB ofrece:

- API Key v3 corta;
- API Read Access Token largo tipo `eyJ...`.

Oberiz fue ajustado para soportar **ambos formatos**.

Lógica:

```text
si empieza por eyJ
    usar Authorization: Bearer
si no
    usar ?api_key=...
```

Esto evita obligar al usuario a saber qué tipo de credencial tiene.

---

## 10. Cliente torrent

Primer cliente soportado: **qBittorrent**

Oberiz no implementará un motor BitTorrent propio.

Se conectará a qBittorrent mediante Web API HTTP/HTTPS.

Configuración prevista:

```text
Host/IP
Puerto
Usuario
Contraseña
HTTPS sí/no
```

Ejemplos:

```text
127.0.0.1:8080
qbittorrent:8080
192.168.1.50:8080
```

qBittorrent puede estar:

- en el mismo servidor;
- en otro contenedor;
- en otro equipo;
- en un NAS.

---

## 11. qBittorrent-nox

Se concluyó que no hace falta empaquetar qBittorrent dentro de Oberiz.

En Linux puede usarse `qbittorrent-nox`.

Arquitectura:

```text
Oberiz
  │
  │ Web API
  ▼
qBittorrent / qbittorrent-nox
```

La interfaz visible será la de Oberiz.

qBittorrent quedará como backend de descarga.

---

## 12. Futuro soporte de otros clientes

La arquitectura se diseñará con una abstracción común.

Conceptualmente:

```text
DownloadClient
├── add_torrent()
├── add_magnet()
├── pause()
├── resume()
├── delete()
├── status()
├── files()
├── trackers()
└── set_category()
```

Clientes futuros:

- Transmission
- Deluge
- otros

---

## 13. Stack tecnológico definitivo inicial

### Backend

**Rust**

Motivos:

- rendimiento;
- bajo consumo de RAM;
- concurrencia;
- seguridad de memoria;
- buen manejo de filesystem;
- binario compacto;
- adecuado para NAS/servidores;
- arquitectura robusta a largo plazo.

Dependencias actuales/principales:

```text
axum
tokio
serde
serde_json
tower-http
tracing
tracing-subscriber
sqlx
anyhow
reqwest
```

### Frontend

Pendiente de iniciar.

Elegido:

```text
React
TypeScript
Vite
```

### Base de datos

Elegida: **SQLite**

Motivos:

- suficiente para el volumen esperado;
- cero administración;
- ideal para servidor doméstico;
- fácil backup;
- fácil Docker;
- archivo único;
- rendimiento suficiente.

Se activó:

- WAL;
- foreign keys.

---

## 14. Entorno de desarrollo

Sistema actual:

```text
Windows
VS Code
PowerShell
```

Ruta local del proyecto:

```text
C:\Users\anjelohe\OneDrive\Documentos\Oberiz
```

Estructura actual aproximada:

```text
Oberiz/
├── backend/
├── frontend/         # pendiente de iniciar
├── config/
│   └── indexers/
│       └── custom/
├── docs/
└── ...
```

Nota: OneDrive funciona, aunque a largo plazo puede ser preferible mover el proyecto a una carpeta como:

```text
C:\GitHub\Oberiz
```

para evitar sincronización de `target/`, `node_modules/`, etc.

---

## 15. Instalación Rust realizada

Rust quedó instalado correctamente mediante rustup.

Versiones comprobadas:

```text
rustc 1.98.1
cargo 1.98.1
rustup 1.29.1
```

También se instalaron los prerrequisitos MSVC/Visual Studio necesarios porque Rust en Windows necesitaba `link.exe`.

Después de instalar Visual C++ Build Tools, el proyecto compiló correctamente.

---

## 16. Backend inicial funcionando

Se creó:

```text
backend/Cargo.toml
backend/src/main.rs
```

Servidor:

```text
Axum + Tokio
```

Puerto elegido:

```text
2032
```

Healthcheck actual:

```text
GET /api/health
```

Respuesta:

```json
{
  "status": "ok",
  "name": "Oberiz",
  "version": "0.1.0",
  "database": "ok"
}
```

Esto confirma:

- backend vivo;
- Rust operativo;
- Axum operativo;
- SQLite conectado.

---

## 17. SQLite realizado

Se creó:

```text
backend/oberiz.db
```

También pueden existir durante ejecución:

```text
oberiz.db-wal
oberiz.db-shm
```

Se corrigió la conexión para crear la base automáticamente:

```rust
.create_if_missing(true)
```

Se activó:

```text
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;
```

---

## 18. Migración inicial realizada

Existe una migración inicial similar a:

```text
backend/migrations/0001_init.sql
```

Tablas actuales:

### settings

```text
key
value
updated_at
```

### movies

```text
id
tmdb_id
title
original_title
year
overview
poster_path
backdrop_path
monitored
library_path
created_at
updated_at
```

Hay restricción única sobre:

```text
movies.tmdb_id
```

y un índice para ese campo.

---

## 19. Módulo de base de datos actual

Existe:

```text
backend/src/db.rs
```

Funciones principales:

- abrir SQLite;
- crear archivo si falta;
- activar WAL;
- activar foreign keys;
- ejecutar migraciones.

---

## 20. Settings implementado

Existe:

```text
backend/src/settings.rs
```

Endpoints actuales:

```text
GET /api/settings
PUT /api/settings
```

Configuración manejada:

```text
tmdb_api_key

qbittorrent_host
qbittorrent_port
qbittorrent_username
qbittorrent_password
qbittorrent_https

movies_path
downloads_path
custom_indexers_path
```

---

## 21. Seguridad de settings

Se mejoró el diseño para que `GET /api/settings` no devuelva secretos en claro.

En vez de devolver claves y contraseñas, devuelve:

```json
{
  "tmdb_api_key_set": true,
  "qbittorrent_password_set": true
}
```

El `PUT` usa `Option<String>` para secretos.

Semántica:

```text
null
= conservar el secreto actual

""
= borrar el secreto

"nuevo_valor"
= sustituirlo
```

Esto será útil cuando exista la interfaz web.

---

## 22. Valores actuales de prueba

Se han usado para pruebas:

```text
qBittorrent host: 127.0.0.1
qBittorrent port: 8080
qBittorrent username: admin
qBittorrent https: false

movies path: D:\Peliculas
downloads path: D:\Descargas

custom indexers:
..\config\indexers\custom
```

Las contraseñas y tokens reales no deben aparecer en el repositorio.

---

## 23. Integración TMDB implementada

Existe:

```text
backend/src/tmdb.rs
```

Endpoint:

```text
GET /api/movies/search?query=...
```

Ejemplo probado:

```text
GET /api/movies/search?query=Matrix
```

Funcionó correctamente con TMDB real.

Devuelve resultados normalizados como:

```json
{
  "tmdb_id": 603,
  "title": "Matrix",
  "original_title": "The Matrix",
  "year": 1999,
  "overview": "...",
  "poster_url": "...",
  "backdrop_url": "...",
  "vote_average": 8.2
}
```

---

## 24. Compatibilidad TMDB API Key / Token

Inicialmente la búsqueda usaba solo `?api_key=`.

Luego se probó Bearer.

Finalmente se decidió soportar ambos formatos automáticamente.

Esto debe mantenerse tanto en:

```text
tmdb.rs
```

como en:

```text
movies.rs
```

---

## 25. Biblioteca de películas implementada

Existe:

```text
backend/src/movies.rs
```

Endpoints actuales:

```text
POST /api/movies
GET  /api/movies
GET  /api/movies/{id}
```

---

## 26. Primera versión de POST /api/movies

Inicialmente se enviaba manualmente:

```json
{
  "tmdb_id": 603,
  "title": "Matrix",
  "original_title": "The Matrix",
  "year": 1999,
  "overview": "...",
  "poster_path": "...",
  "backdrop_path": null,
  "monitored": true,
  "library_path": null
}
```

Esto funcionó y se guardó Matrix en SQLite.

---

## 27. Mejora de POST /api/movies

Se cambió el diseño para que el frontend NO envíe metadata.

Ahora basta con:

```json
{
  "tmdb_id": 157336,
  "monitored": true,
  "library_path": null
}
```

Oberiz hace automáticamente:

```text
tmdb_id
  ↓
consulta TMDB
  ↓
obtiene metadata real
  ↓
guarda SQLite
```

Esto evita:

- datos inconsistentes;
- títulos incorrectos;
- posters erróneos;
- duplicación de lógica en frontend.

---

## 28. Películas probadas

Se utilizó:

```text
Matrix
TMDB ID: 603
```

La primera inserción se hizo manualmente.

Después se probó:

```text
Interstellar
TMDB ID: 157336
```

con el nuevo flujo automático desde TMDB.

Resultado: **guardado correctamente.**

---

## 29. Control de duplicados

`movies.tmdb_id` es UNIQUE.

Si se intenta añadir de nuevo una película ya existente:

```text
HTTP 409 CONFLICT
```

Mensaje esperado:

```text
La película ya existe en Oberiz
```

---

## 30. UTF-8

Durante las pruebas con PowerShell apareció un problema con caracteres acentuados en JSON.

Se resolvió enviando el body como bytes UTF-8:

```powershell
$json = $body | ConvertTo-Json
$utf8 = [System.Text.Encoding]::UTF8.GetBytes($json)

Invoke-RestMethod `
    -ContentType "application/json; charset=utf-8" `
    -Body $utf8
```

Esto es importante porque Oberiz debe manejar correctamente español, tildes, ñ, idiomas internacionales y títulos Unicode.

---

## 31. Estado actual de endpoints

Actualmente deberían existir:

```text
GET  /api/health

GET  /api/settings
PUT  /api/settings

GET  /api/movies/search?query=...

POST /api/movies
GET  /api/movies
GET  /api/movies/{id}
```

---

## 32. Próximo paso inmediato

Antes de qBittorrent se decidió completar las operaciones básicas de películas.

Pendiente inmediato:

```text
PUT    /api/movies/{id}
DELETE /api/movies/{id}
```

El `PUT` debería modificar solo información local:

```text
monitored
library_path
```

No debería permitir alterar arbitrariamente metadata que viene de TMDB.

---

## 33. Integración qBittorrent prevista

Después de terminar CRUD básico de películas:

```text
POST /api/settings/test-qbittorrent
```

Debe:

1. leer host/puerto/user/password;
2. construir URL HTTP/HTTPS;
3. hacer login contra qBittorrent;
4. comprobar respuesta;
5. obtener versión;
6. devolver estado.

Después se implementará un módulo:

```text
backend/src/qbittorrent.rs
```

---

## 34. Funciones previstas de qBittorrent

Primera fase:

```text
login
version
transfer info
torrent list
add magnet
add torrent
pause
resume
delete
```

Endpoints Oberiz previstos:

```text
GET    /api/downloads
POST   /api/downloads/magnet
POST   /api/downloads/torrent
POST   /api/downloads/{hash}/pause
POST   /api/downloads/{hash}/resume
DELETE /api/downloads/{hash}
```

---

## 35. Pantalla de descargas futura

Oberiz mostrará su propia UI y no dependerá visualmente de la WebUI de qBittorrent.

Ejemplo:

```text
Downloads

Dune Part Two
████████████████░░░░ 82 %
45.3 MB/s
ETA 4 min

Silo S02E04
██████████░░░░░░░░░░ 43 %
12.1 MB/s
```

---

## 36. Indexer Engine previsto

Después de qBittorrent se empezará:

```text
backend/src/indexers/
```

Funciones:

- leer YAML;
- validar definición;
- cargar indexadores;
- login;
- cookies;
- búsquedas;
- categorías;
- parseo HTML;
- normalización de resultados.

Endpoints previstos:

```text
GET  /api/indexers
POST /api/indexers/reload
POST /api/search/movie/{id}
```

---

## 37. Release Engine previsto

Será una de las piezas más complejas.

Ejemplo:

```text
Dune.Part.Two.2024.2160p.UHD.BluRay.REMUX.DV.HDR10.HEVC.TrueHD.Atmos
```

Debe extraer:

```text
movie
year
2160p
BluRay
REMUX
Dolby Vision
HDR10
HEVC
TrueHD Atmos
grupo
idioma
```

Para series:

```text
The.Last.of.Us.S02E04.2160p.MAX.WEB-DL.DDP5.1.Atmos.DV.HDR.H.265-NTb
```

Debe extraer:

```text
serie
temporada
episodio
resolución
source
codec
audio
HDR
grupo
```

---

## 38. Scoring previsto

Sistema conceptual inicial:

```text
2160p          +100
REMUX           +80
BluRay          +60
WEB-DL           +40
Dolby Vision     +30
HDR10            +20
Castellano       +50
Dual             +30
CAM             -500
TS              -500
```

Más adelante:

- perfiles;
- custom formats;
- codecs;
- idiomas;
- grupos;
- tamaños;
- upgrades.

---

## 39. Importer previsto

Cuando qBittorrent termine:

```text
Torrent completado
   ↓
detectar película asociada
   ↓
validar archivo
   ↓
mover o hardlink
   ↓
renombrar
   ↓
actualizar DB
   ↓
marcar disponible
```

Funciones:

- mover;
- hardlink;
- rename;
- verificar extensiones;
- evitar extras no deseados;
- manejar carpetas;
- historial.

---

## 40. Modelo de datos futuro

Tablas previstas además de `settings` y `movies`:

### indexers

```text
id
name
definition_path
enabled
type
created_at
updated_at
```

### releases

```text
id
movie_id
indexer_id
title
download_url
magnet
size
seeders
leechers
quality
source
codec
hdr
audio
language
score
created_at
```

### downloads

```text
id
movie_id
release_id
client
torrent_hash
status
progress
download_path
created_at
updated_at
```

### history

```text
id
media_type
media_id
event_type
message
created_at
```

Más adelante:

```text
series
seasons
episodes
profiles
custom_formats
notifications
```

---

## 41. Series

Las series NO forman parte del primer MVP.

Se añadirán después de completar el pipeline de películas.

Funciones futuras:

- buscar series;
- temporadas;
- episodios;
- monitorización;
- SxxExx;
- multi-episodio;
- packs de temporada;
- upgrades;
- RSS.

---

## 42. Entidad común Media

A medio plazo se pretende evitar duplicar lógica.

Modelo:

```text
Media
├── Movie
└── Series
```

Pipeline:

```text
Media
  ↓
Wanted
  ↓
Search
  ↓
Release
  ↓
Score
  ↓
Download
  ↓
Import
```

---

## 43. Frontend previsto

Aún no iniciado.

Se usará:

```text
React
TypeScript
Vite
```

Pantallas iniciales:

```text
Dashboard
Movies
Downloads
Indexers
History
Settings
```

Después:

```text
Series
Calendar
Wanted
Activity
Profiles
Custom Formats
Notifications
System
```

---

## 44. UI de Settings prevista

Bloques:

```text
TMDB
- credencial
- probar conexión

qBittorrent
- host
- port
- username
- password
- HTTPS
- probar conexión

Paths
- películas
- descargas
- custom indexers
```

Los secretos no se mostrarán en claro.

---

## 45. Configuración del servidor

Puerto actual hardcodeado:

```text
2032
```

Más adelante deberá ser configurable mediante settings, archivo o variable de entorno.

Posible configuración futura:

```toml
[server]
host = "0.0.0.0"
port = 2032
```

---

## 46. Docker previsto

Todavía no implementado.

Objetivo:

```text
docker compose up -d
```

Oberiz podrá conectarse a qBittorrent externo.

También se podrá ofrecer un compose opcional:

```text
oberiz
qbittorrent
```

sin obligar a usarlo.

---

## 47. Seguridad

Principios acordados:

- no subir tokens;
- no subir contraseñas;
- no subir cookies;
- no subir trackers custom privados;
- no devolver secretos desde GET settings;
- usar variables/configuración local;
- soporte HTTPS mediante proxy inverso o configuración futura;
- posible cifrado local de secretos más adelante.

---

## 48. .gitignore previsto

Como mínimo:

```gitignore
backend/target/
frontend/node_modules/
frontend/dist/

.env
.env.*

*.db
*.db-wal
*.db-shm

config/secrets*
config/indexers/custom/*
```

---

## 49. GitHub

Oberiz se publicará como código libre.

Repositorio previsto:

```text
README.md
LICENSE
CONTRIBUTING.md
SECURITY.md
docs/
backend/
frontend/
config/
```

Los custom privados no se incluirán.

---

## 50. Financiación futura

Decisión:

- proyecto gratuito;
- código abierto;
- sin monetización obligatoria.

Se podrían aceptar donaciones, GitHub Sponsors o patrocinios voluntarios.

Esto no cambia el carácter open source.

Si hubiera ingresos recurrentes significativos, se revisaría la parte fiscal en España.

---

## 51. MVP definido

El MVP de películas debe conseguir este flujo completo:

```text
Buscar película en TMDB
      ↓
Añadir película
      ↓
Buscar releases
      ↓
Elegir release
      ↓
Enviar a qBittorrent
      ↓
Monitorizar descarga
      ↓
Importar
      ↓
Mover / hardlink
      ↓
Renombrar
      ↓
Biblioteca
```

Cuando esto funcione de extremo a extremo, Oberiz ya será usable.

---

## 52. Roadmap actualizado

### Fase 1 — Base técnica

**HECHO**

- Rust
- Axum
- Tokio
- puerto 2032
- healthcheck
- SQLite
- WAL
- migraciones
- AppState
- Reqwest

### Fase 2 — Settings

**HECHO**

- GET settings
- PUT settings
- persistencia SQLite
- secretos no devueltos
- rutas
- TMDB
- qBittorrent config

Pendiente:

- test TMDB dedicado
- test qBittorrent

### Fase 3 — TMDB

**HECHO**

- búsqueda de películas
- metadata
- español
- poster
- backdrop
- ratings
- API key v3
- Bearer token

### Fase 4 — Movies

**EN PROGRESO**

Hecho:

- POST
- GET list
- GET detail
- metadata automática desde TMDB
- control de duplicados

Pendiente:

- PUT movie
- DELETE movie
- refresh metadata
- marcar monitorizada/no monitorizada

### Fase 5 — qBittorrent

**SIGUIENTE GRAN BLOQUE**

- test conexión
- login
- versión
- lista de torrents
- añadir magnet
- añadir torrent
- estado
- pause
- resume
- delete

### Fase 6 — Indexers

Pendiente:

- parser YAML
- cargar custom
- buscar
- normalizar resultados
- categorías
- cookies/login
- compatibilidad progresiva con Prowlarr/Cardigann

### Fase 7 — Releases

Pendiente:

- parser
- quality
- source
- codec
- HDR
- audio
- idiomas
- scoring

### Fase 8 — Downloads tracking

Pendiente:

- relacionar movie/release/hash
- progreso
- estados
- historial

### Fase 9 — Importer

Pendiente:

- mover
- hardlink
- rename
- validar
- actualizar biblioteca

### Fase 10 — Frontend

Pendiente:

- React
- Vite
- dashboard
- movies
- search
- settings
- downloads
- indexers

### Fase 11 — Series

Pendiente.

### Fase 12 — Automatización avanzada

Pendiente:

- RSS
- scheduler
- upgrades
- custom formats
- perfiles
- notificaciones

---

## 53. Próxima tarea concreta

La siguiente tarea recomendada es:

```text
PUT /api/movies/{id}
DELETE /api/movies/{id}
```

Después:

```text
POST /api/settings/test-qbittorrent
```

y arrancar la integración real con qBittorrent.

---

## 54. Estado actual resumido

Hoy Oberiz ya puede:

```text
✓ arrancar backend
✓ responder healthcheck
✓ usar SQLite
✓ guardar settings
✓ ocultar secretos en GET
✓ conectarse a TMDB
✓ buscar películas
✓ soportar API Key y Bearer
✓ guardar películas
✓ obtener metadata automáticamente
✓ listar películas
✓ consultar película individual
✓ evitar duplicados
```

Todavía no puede:

```text
✗ editar película
✗ borrar película
✗ probar qBittorrent
✗ listar descargas
✗ buscar en indexadores
✗ puntuar releases
✗ importar archivos
✗ gestionar series
✗ mostrar frontend
```

---

## 55. Principio de desarrollo

Se mantiene la idea de avanzar por capas.

Orden de prioridad:

1. que funcione;
2. que sea simple;
3. que sea mantenible;
4. que sea seguro;
5. que pueda evolucionar;
6. añadir tests antes de que crezca demasiado.

No se intentará replicar de golpe años de desarrollo de Radarr, Sonarr y Prowlarr.

---

## 56. Decisiones cerradas

```text
Nombre                Oberiz
Backend               Rust
Framework             Axum
Runtime               Tokio
HTTP client           Reqwest
DB                    SQLite
DB access             SQLx
Frontend              React + TypeScript + Vite
Metadata              TMDB
Torrent inicial       qBittorrent
Puerto Oberiz         2032
Custom indexers       fuera de GitHub
Licencia              pendiente de cerrar
Color principal       oscuro + verde eléctrico
Proyecto              gratuito y open source
```

---

## 57. Archivos principales actuales

```text
backend/
├── Cargo.toml
├── oberiz.db
├── migrations/
│   └── 0001_init.sql
└── src/
    ├── main.rs
    ├── db.rs
    ├── settings.rs
    ├── tmdb.rs
    └── movies.rs
```

---

## 58. Objetivo final

Oberiz aspira a convertirse en una única aplicación para:

```text
Películas
Series
Indexadores
Descargas
RSS
Scoring
Perfiles
Custom Formats
Importación
Historial
Calendario
Notificaciones
qBittorrent
Transmission
Deluge
```

Todo desde una única interfaz web y con una instalación simple.

---

> **Nota histórica (cierre 0.8.4):** el bloque siguiente documenta la transición desde 0.7.0. El estado vigente del proyecto está al inicio de esta memoria.

---

## 59. Evolución consolidada 0.7.0–0.8.4

### 0.7.0 — Series, temporadas y categorías

- selector real de categorías de qBittorrent y sus rutas de guardado;
- categoría predeterminada por Quality Profile;
- temporadas y episodios desde TMDB, monitor modes y perfiles heredados/override;
- búsquedas de packs de temporada y SxxExx;
- automatización e importación conscientes de episodios;
- migración `0008_series_seasons_episodes.sql`.

### 0.7.1–0.7.3 — Perfiles y metadata de qBittorrent

- selección rápida de Quality Profile al añadir y editar películas;
- categoría vacía como `Uncategorized` real, sin fallback implícito;
- perfil predeterminado único para Movies y Series con protección de borrado/desactivación (`0009_default_quality_profiles.sql`);
- `setCategory`, `addTags`, verificación posterior y persistencia inmediata de `qb_hash`.

### 0.7.4–0.7.5 — Flujo global y protección de duplicados

- modal global Add Media y pulido de bibliotecas;
- resúmenes reales de temporadas/episodios y panel Downloads ampliado;
- alta de magnets idempotente por infohash, también segura frente a carreras.

### 0.8.0 — Biblioteca y API

- estados de biblioteca basados en archivos reales;
- rescan, upgrades por calidad instalada y Calendar;
- importer ampliado y Public API v1;
- migración `0010_library_calendar_public_api.sql`.

### 0.8.1–0.8.2 — Dashboard y clave pública

- Dashboard alimentado por descargas, biblioteca, actividad, indexadores y próximos episodios;
- Show, Hide, Copy y Regenerate para API key;
- clave recuperable desde Settings y enmascarada por defecto.

### 0.8.3 — Complete Series

`Accept complete series` pasa a la automatización: para una biblioteca de serie vacía con varios episodios emitidos pendientes, se intenta primero una release de serie completa. Si no es válida, continúa con pack de temporada y después episodio.

### 0.8.4 — Estabilización

- pruebas automáticas de parser/scoring y de la prioridad Complete Series → Season Pack → Episode;
- endpoint y pantalla de diagnóstico sin secretos;
- versiones alineadas en backend, frontend y UI;
- documentación consolidada.

### 0.9.0 — Automatización continua RSS

- migración `0011_rss_automation.sql` con estado de sincronización y releases procesadas por indexador;
- URL RSS opcional y explícita dentro de la configuración de cada indexador;
- lectura incremental de RSS y Atom, con deduplicación persistente por `indexer_id` + GUID;
- matching contra películas y series monitorizadas, reutilizando parser, perfiles y scoring existentes;
- prioridad de serie completa, pack de temporada y episodio según las reglas del perfil;
- protección contra trabajos equivalentes activos, registro de cada decisión y grab mediante el flujo existente de qBittorrent;
- controles de activación, intervalo y sincronización manual en Settings; endpoints `GET /api/rss/status` y `POST /api/rss/run`.

### Integración nativa Cinetta — posterior a 0.9.0

- Public API v1 ampliada con `GET /api/v1/quality-profiles`, autenticado con `X-Api-Key`, para que Cinetta lea los perfiles activos sin depender de rutas internas de Oberiz. Cada perfil expone `request_quality` (`standard` o `4k`), configurado mediante un check explícito en Oberiz para que los clientes nativos nunca mezclen destinos 1080p y 4K.
- Las solicitudes públicas aceptan `client_name` y `client_request_id`; juntos convierten el alta en idempotente y permiten que Cinetta reintente una aprobación sin crear otra descarga.
- Las peticiones siguen conservando el perfil, el medio, TMDb y el estado real (`pending`, `searching`, `downloading`, `monitoring`, `available`).
- Cinetta opera exclusivamente mediante la API nativa y deja en Oberiz las decisiones de rutas, categorías de qBittorrent, etiquetas, importación y reglas de calidad.
- La capa Radarr/Sonarr queda como compatibilidad opcional para Overseerr y clientes equivalentes. Está aislada bajo `overseerr.compat_enabled`, no altera el contrato nativo ni requiere que Cinetta emule APIs ajenas.
- La integración se verificó mediante compilación de Oberiz y pruebas de proveedores/peticiones de Cinetta.
