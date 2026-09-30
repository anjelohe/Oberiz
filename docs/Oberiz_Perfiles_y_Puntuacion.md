# Perfiles de calidad, puntuación y prioridad de indexadores

Esta guía explica cómo Oberiz decide qué release mostrar primero. Los perfiles no descargan automáticamente una release por estar arriba: sirven para aceptar, rechazar y ordenar los resultados disponibles.

## 1. Aceptación: el filtro obligatorio

Antes de comparar puntuaciones, Oberiz evalúa cada release contra el perfil seleccionado. Una release rechazada siempre queda por detrás de una aceptada, incluso cuando el perfil activa la prioridad de indexador.

Las causas habituales de rechazo son:

- coincidencia insuficiente con el título;
- año incompatible cuando el release declara uno alejado más de un año;
- tipo de medio incorrecto, por ejemplo una temporada al buscar una película;
- temporada explícita distinta de la solicitada;
- seeds por debajo del mínimo;
- tamaño por debajo o por encima de los límites del perfil;
- término bloqueado;
- resolución, fuente o idioma no permitidos.

Los términos bloqueados se comparan como tokens de release. Por ejemplo, `CAM` bloquea `CAM.1080p`, pero no bloquea un título que contenga esas letras dentro de una palabra.

## 2. Cómo se calcula la puntuación

Toda release aceptada recibe una puntuación total formada por:

```text
coincidencia de título
+ puntuación del perfil
+ bonus de seeds (máximo 60)
```

La puntuación del perfil suma los valores configurados en:

- resolución;
- fuente;
- códec;
- HDR;
- audio;
- términos preferidos, por ejemplo `PROPER` o `REPACK`;
- idioma, según el perfil de idioma asociado;
- bonus de `+35` para packs de series cuando se activa **Prefer season/complete packs**.

Los valores no son niveles obligatorios: son preferencias numéricas. Una dimensión con valores configurados puede además rechazar elementos no permitidos, como una resolución o fuente desmarcada. Las dimensiones opcionales —códec, HDR y audio— solo suman cuando se reconocen y existen en el perfil.

## 3. Límites de tamaño en series

Los perfiles de series permiten dos máximos:

- **Max episode size MB** se aplica a capítulos explícitos como `S03E07`, `3x07`, `Episode` o `Episodio`.
- **Max season/pack size MB** se aplica a `S03`, `Temporada 3`, `PACK` o `Complete` cuando no hay un marcador de capítulo.

Si el segundo campo está vacío, Oberiz usa el máximo general del episodio también para packs. Esto conserva el comportamiento previo hasta que se configure un máximo independiente.

## 4. Orden normal: la calidad manda

Con **Prioritize indexer** desactivado, los resultados se ordenan así:

1. Releases aceptadas antes que rechazadas.
2. Mayor puntuación total.
3. Menor prioridad numérica del indexador.
4. Mayor número de seeds.

En este modo, la prioridad del indexador es un desempate. Es el modo recomendado cuando el perfil de calidad representa completamente tus preferencias.

## 5. Orden con Prioritize indexer

Cada perfil de calidad puede activar **Prioritize indexer**. Al hacerlo, el orden cambia a:

1. Releases aceptadas antes que rechazadas.
2. Menor prioridad numérica del indexador.
3. Mayor puntuación total dentro de ese indexador.
4. Mayor número de seeds.

Así puedes dar preferencia a un indexador que normalmente publica mejores copias, manteniendo las reglas de aceptación del perfil. Una release que incumpla resolución, idioma, tamaño, seeds, términos bloqueados u otra regla sigue siendo rechazada y no puede ganar por la prioridad.

## 6. Configurar la prioridad

La prioridad se configura al editar cada indexador mediante **Search priority** y se muestra en la tabla de Indexers.

- El valor predeterminado es `100`.
- Un número menor es más preferente: `10` se consulta y ordena antes que `100`.
- No añade puntos a una release; es una regla de ordenación.

Ejemplo: si un indexador tiene prioridad `10`, otro `100` y el perfil activa **Prioritize indexer**, se muestran primero las releases aceptadas del indexador `10`. Dentro de ese grupo, Oberiz ordena por la puntuación de calidad y después por seeds.

## 7. Recomendación práctica

- Ajusta primero las reglas de aceptación y los puntos de calidad para impedir formatos o idiomas no deseados.
- Usa la prioridad normal para desempates neutrales.
- Activa **Prioritize indexer** solo en los perfiles donde confíes especialmente en el criterio editorial de uno o varios indexadores.
- Mantén el check desactivado en perfiles donde quieras comparar estrictamente la mejor puntuación disponible entre todas las fuentes.

Las definiciones YAML, direcciones, credenciales y notas de trackers privados no forman parte de esta guía ni del repositorio.
