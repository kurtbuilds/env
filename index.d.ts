export declare function ok(): void;
declare type ENV_TYPE = {
    opt: Record<string, string | undefined>;
    num: Record<string, number>;
    ok: () => void;
} & Record<string, string>;
export declare const ENV: ENV_TYPE;
export {};
