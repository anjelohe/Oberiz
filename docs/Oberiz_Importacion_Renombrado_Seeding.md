# Oberiz — Importación, renombrado y políticas de seed

## 1. Problema

Un tracker privado puede exigir compartir X días, alcanzar ratio X o ambas cosas.

Mover/renombrar directamente el archivo que qBittorrent está sembrando puede romper el torrent.

## 2. Solución recomendada

### Hardlink
Si Downloads y Library están en el mismo filesystem:

```text
Downloads:
Avatar.2009.EXTENDED.1080p.BluRay.x265-RARB.mkv

Library:
Avatar (2009)/Avatar (2009) - 1080p BluRay x265.mkv
```

Ambos nombres apuntan a los mismos datos físicos.

Ventajas:
- qBittorrent sigue sembrando;
- biblioteca limpia;
- no duplica espacio;
- borrar el original tras cumplir seed no elimina la biblioteca.

### Copy fallback
Si están en filesystems distintos:
- copiar;
- conservar original hasta cumplir seed.

## 3. Regla

```text
if seeding_required:
    hardlink_or_copy
else:
    move_or_hardlink
```

## 4. Seed policy por indexador

```json
{
  "min_seed_time_minutes": 10080,
  "min_ratio": 1.0,
  "requirement_mode": "both",
    "cleanup_mode": "remove_torrent_and_original"
}
```

Modos:
- time;
- ratio;
- either;
- both;
- manual.

Cleanup:
- remove_torrent_keep_files;
- remove_torrent_and_original;
- manual;
- never.

## 5. Trackers privados

Si el indexador es privado y no tiene política:

```text
NO AUTO DELETE
```

## 6. Datos qBittorrent disponibles

Oberiz ya recibe:
- ratio;
- seeding_time;
- completion_on;
- uploaded;
- downloaded;
- state;
- category;
- tags;
- tracker/source.

Falta persistir:

```text
download hash ↔ release ↔ media ↔ indexer
```

## 7. Plantillas de renombrado

Películas:

```text
{Title} ({Year}) - {Resolution} {Source} {Codec}
```

Series:

```text
{SeriesTitle} - S{Season:00}E{Episode:00} - {EpisodeTitle} - {Resolution} {Source} {Codec}
```

## 8. Ciclo de importación

```text
qBittorrent Completed
→ DownloadJob Completed
→ identificar archivos
→ ignorar samples/extras
→ determinar Movie/Episode
→ calcular nombre final
→ hardlink/copy
→ registrar MediaFile
→ Available
→ seguir seed
→ cumplir SeedPolicy
→ cleanup seguro
```


---

## Routing de qBittorrent por perfil

La categoría de qBittorrent pertenece al Quality Profile porque representa el destino/flujo de ese tipo de contenido.

Ejemplos:

```text
Movie 1080p -> peliculasWD19
Movie 4K    -> peliculasWD19_4k
Series 1080p -> seriesWD19
Series 4K    -> seriesWD19_4k
```

El tracker/indexador se recomienda como etiqueta:

```text
qBittorrent Tags: [tracker]
```

Los tags se resuelven al hacer Grab y se guardan en `DownloadJob` para que el reseed futuro conserve el routing original.

Oberiz mantiene una única etiqueta técnica reservada `_oberiz_job_<id>` para correlación con qBittorrent.


---

## Series episode-aware desde 0.7.0

El importer reconoce `SxxExx` y rangos multi-episodio.

Cuando identifica un episodio:
- registra `media_files`;
- marca `series_episodes.has_file = true`;
- crea relación en `episode_files`.

Template Series recomendado:

```text
{Title} - S{Season:00}E{Episode:00} - {EpisodeTitle}
```

Los packs con varios archivos se importan archivo a archivo cuando los filenames contienen numeración reconocible.

La Category de qBittorrent continúa determinando dónde descarga qBittorrent. La biblioteca final sigue usando `paths.series`.
