import type { Url } from "next/dist/shared/lib/router/router";
import Image, { StaticImageData } from "next/image";
import Link from "next/link";
import { AspectRatio } from "../../components";
import { Icons } from "../../icons";

interface AuthLayoutImageProps {
  imagesrc: StaticImageData;
  alt?: string;
  linkToImage?: Url;
  photographer?: string;
  photographerId?: Url;
}
export function AuthLayoutImage({
  imagesrc,
  alt,
  photographer,
  photographerId,
  linkToImage,
}: AuthLayoutImageProps) {
  return (
    <AspectRatio ratio={16 / 9}>
      <Image
        src={imagesrc}
        alt={alt ?? "Image"}
        fill
        className="absolute inset-0 object-cover"
        priority
        sizes="(max-width: 768px) 100vw, (max-width: 1200px) 50vw, 33vw"
      />
      <div className="absolute inset-0 bg-gradient-to-t from-background to-background/60 md:to-background/40" />
      <Link
        href="/"
        className="flex absolute left-8 top-6 z-20 items-center text-lg font-bold tracking-tighter"
      >
        <Icons.chevronsRight className="mr-2 h-6 w-6" aria-hidden="true" />
        <span>Velocity</span>
      </Link>
      <div className="absolute bottom-6 left-8 z-20 line-clamp-1 text-base">
        Photo by{" "}
        <a href={photographerId as string} className="hover:underline">
          {photographer ?? "unknown"}
        </a>
        {" on "}
        <a href={linkToImage as string} className="hover:underline">
          Unsplash
        </a>
      </div>
    </AspectRatio>
  );
}
