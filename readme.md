
Isometric renderer for a voxel based world than allows a slice of the world to be scene at a time

some workers that can be controlled via mouse
The world has randomized craters created on startup and uses calculated lighting to delineate cube faces


Controls:

Mouse to select cube or worker

scroll wheel -> go up and down through layers
wasd -> pan over x and y directions
h -> return to home view
x -> delete the hovered cube
n -> night mode
left click -> select worker

When a worker is selected (currently no indication of this)
l -> place a lantern above the highlighted square
k -> dig up the hovered cube
p -> plough the hovered cube (doesn't have much effect)
left click -> move worker to square


KNOWN ISSUES:
workers may spawn in crater and be stuck
mouse selection has rounding errors which makes it feel slightly off, particularly when selecting cubes many layers
below the view window top, this can be fixed by scrolling down to a closer layer
