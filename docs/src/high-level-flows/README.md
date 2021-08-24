# High level flows

The following use cases will help contextualize and understand the overall flow between the different applications making up the Legion engine and its clients.

## Game production / development

The Legion engine provides a full production pipeline for the creation of games and interactive environments.

In these game production scenarios, the user is part of the game development team in a company that is a client of Legion Labs.

### Startup sequence

The user starts up the Legion Labs editor.

> The editor could be a stand-alone application (that was previously installed), or accessible through a web browser. To be determined...

If accessing for the first time, the user must first select which project to work on (the company could be working on several titles), and then which specific world within the project to open. If returning, the settings will default to the values from the previous session.

> If a project contains a single world, or has a base root world, it will be selected by default.

The editor will then open up and be ready for the user to browse and edit the world's contents.

![Editor session](figures/editor-session.png)

### Collaboration

By choosing to use shared virtual workspaces, users have the advantage of always seeing the latest changes from their team.

### Importing source data

Many types of resources will be edited outside of the Legion Labs editor.

This includes:
* Textures
* Visual meshes
* Animations
* Sound

These will be created and edited in dedicated applications, such as Maya or 3ds Max for visual meshes, and then imported into the source-control system.

A DCC exporter (one per resource type) will convert the data to an offline format that is usable in the editor.

### Scripting game logic

!todo

### Testing changes

!todo

### Production analytics

!todo

### Outsourcing

How to work with an external localization contractor?

## Game live operations

!todo

### Packaging a release

!todo

### First party submissions

!todo
