# Blend

A little utility I use to create background images from an image.

This command will create a new image, `dst.jpg` by paining the background black (`#000000`) and then painting the image from `src.jpg` on top with an opacity of 0.6.

```console
$ blend --background=#000000 --opacity=0.6 src.jpg dst.jpg
```
