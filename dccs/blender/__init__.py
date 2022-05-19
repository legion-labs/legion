bl_info = {
    "name" : "Legion Blender Plugin",
    "author" : "Legion Labs",
    "description" : "Interoperability between Legion Engine and Blender",
    "blender" : (3, 1, 2),
    "version" : (0, 0, 1),
    "location" : "View 3D panel",
    "warning" : "",
    "category" : "Generic"
}

import sys, os
root_path = os.path.dirname(__file__)
dependencies_path = os.path.join(root_path, 'dependencies')
generated_path = os.path.join(root_path, 'generated')
sys.path.append(root_path)
sys.path.append(dependencies_path)
sys.path.append(generated_path)

import preferences
import operator_pull
import operator_push

def register():
    preferences.register()
    operator_push.register()
    operator_pull.register()

def unregister():
    preferences.unregister()
    operator_push.unregister()
    operator_pull.unregister()