export interface Transport {
  doRequest(key: string, arg: any): Promise<any>;
}

export class FetchTransport implements Transport {
  private url: string;

  constructor(url: string) {
    this.url = url;
  }

  async doRequest(key: string, arg: any): Promise<any> {
    const resp = await fetch(
      `${this.url}/${key}?batch=1&input=${encodeURIComponent(arg || "{}")}`
    );
    // TODO: Error handling
    const body = await resp.json();
    return body[0].result.data;
  }
}
