export type PtySize = {
    rows: number;
    cols: number;
    pixel_width: number;
    pixel_height: number;
};

export type Terminal = {
    id: string;
    command: string;
    args: string[] | null;
    title: string | null;
    size: PtySize;
};
