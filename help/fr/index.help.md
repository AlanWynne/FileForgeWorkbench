<!-- TOPIC: getting_started -->
<!-- TITLE: Premiers pas -->

# Premiers pas avec FileForge Workbench

FileForge Workbench est un environnement d'édition piloté par commandes, inspiré
d'ISPF. Ce parcours couvre l'essentiel pour une première session.

<!-- DRAFT: AI-DRAFTED French help, PENDING NATIVE-SPEAKER REVIEW (CR-NR-103
Task 10). Only the TITLE and body prose are translated; the TOPIC key and the
topic/feature:/line: cross-reference keys stay unlocalised (Req 6.3).
Untranslated topics fall back to the English help/ directory (Req 6.2). Help
content is user-facing rendered markdown (no Fluent escape processing), so
accented letters are written literally -- the runtime-UI-strings exception to the
ASCII-first documentation rule. -->

## Ouvrir un fichier

- Utilisez le menu Fichier, ou tapez un chemin de fichier sur la ligne
  `Command ===>` et appuyez sur Entrée.
- Les fichiers s'ouvrent dans un Contexte d'édition au sein d'un onglet Espace de travail.

## Navigation de base

- **F7 / F8** font défiler d'une page vers le haut / bas (régi par la valeur `SCROLL ===>`).
- **UP** et **DOWN** font défiler selon la valeur SCROLL active.
- **TOP** et **BOTTOM** sautent à la première / dernière ligne.
- **LOCATE n** se déplace à la ligne numéro n.

## Édition

- En mode Édition, tapez directement dans la zone de texte.
- Les **commandes de ligne** de la zone de préfixe (par exemple `D` pour supprimer
  une ligne, `CC` pour copier un bloc) agissent sur des lignes individuelles.
  Voir [Line Commands](line:index).
- **UNDO** et **REDO** annulent et réappliquent les modifications. Voir [Undo](feature:undo).

## Enregistrement

- **SAVE** écrit le tampon actuel. **CANCEL** abandonne les modifications. **END** (F3)
  ferme le Contexte actuel.

## La ligne de commande

- Chaque action est une commande. Les menus, boutons et touches de fonction
  invoquent tous les mêmes commandes que vous pouvez taper sur la ligne `Command ===>`.
- Appuyez sur **F1** à tout moment pour une aide contextuelle sur ce que vous faites.
- **F12** (RETRIEVE) rappelle les commandes précédentes.

## Organiser l'atelier

- Les Espaces de travail sont des onglets; un Espace de travail peut être divisé
  ou détaché dans sa propre fenêtre.
- Voir [Tabs](feature:tabs) et [Docking](feature:docking).

Voir aussi : [Help Index](index), [Function Keys](feature:function_keys).
