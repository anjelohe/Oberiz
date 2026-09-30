# Documentación de Oberiz

Este directorio contiene toda la documentación versionada del proyecto. La raíz del repositorio conserva un único `README.md` orientado a instalar y entender el producto actual.

## Documentos vigentes

- [`Oberiz_Memoria_Actualizada.md`](Oberiz_Memoria_Actualizada.md) — estado funcional, arquitectura, integraciones y pendientes reales de 0.9.0.
- [`Oberiz_Plan_Proyecto.md`](Oberiz_Plan_Proyecto.md) — roadmap vigente desde el cierre operativo de 0.9.0 hasta 1.0.
- [`Oberiz_API_Integraciones.md`](Oberiz_API_Integraciones.md) — contrato actual de Public API v1, perfiles, autenticación, idempotencia y compatibilidad opcional.
- [`openapi.yaml`](openapi.yaml) — el mismo contrato de Public API v1 en formato OpenAPI 3.0, para generar clientes o abrir en Swagger UI/Redoc.
- [`Oberiz_Importacion_Renombrado_Seeding.md`](Oberiz_Importacion_Renombrado_Seeding.md) — importación, nomenclatura, reseed y seed policies.
- [`Oberiz_Perfiles_y_Puntuacion.md`](Oberiz_Perfiles_y_Puntuacion.md) — criterios de aceptación, puntuación de releases y prioridad de indexadores.

La memoria conserva también la cronología completa del desarrollo inicial, claramente separada del estado vigente para que no se pierdan decisiones ni hitos anteriores.

## Cambios públicos recientes

- Búsquedas Cardigann más tolerantes: reintento sin año y con título original, además de lectura HTML que evita cabeceras y filas vacías.
- Perfiles de calidad: clonación, filtrado por temporada, preferencia por packs y máximos independientes para capítulos y temporadas/packs.
- La prioridad de indexador es visible en la lista de configurados; cada perfil puede activar **Prioritize indexer** para preferirlo entre releases aceptadas.

Los YAML de trackers, nombres de trackers privados, credenciales, URL locales y notas operativas se excluyen deliberadamente de esta documentación y del historial de Git.

## Documentación y Git

La documentación de producto se mantiene en Git dentro de `docs/`. No se versionan secretos, bases de datos SQLite, builds, dependencias ni indexadores privados; esas exclusiones están definidas en `.gitignore`.
