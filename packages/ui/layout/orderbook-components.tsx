export const Orderbook = () => {
  return (
    <div className="flex flex-col w-[300px] overflow-hidden border rounded-lg">
      <div className="flex items-center justify-between border-b px-3 py-2 text-xs">
        <p className="font-medium">Price (USDC)</p>
        <p className="font-medium text-right">Total (SOL)</p>
      </div>
      <OrderbookContent />
    </div>
  );
};

const OrderbookContent = () => {
  const price = 100;
  const total = 10;
  const ltsPrice = 99;
    return (
      <div className="flex flex-col grow overflow-y-hidden">
        <div className="flex flex-col h-full grow overflow-x-hidden">
          <div className="flex flex-col grow overflow-y-scroll max-h-[42rem] font-sans snap-y  ">
            <div className="flex flex-col-reverse">
              {/* {orderbook.askDepth.map(([price, total]) => ( */}
                <OrderbookEntry
                  key={`ask-${price}`}
                  price={price.toFixed(2)}
                  total={total}
                  type="ask"
                />
              {/* ))} */}
            </div>
            <OrderbookMidpoint price={ltsPrice} />
            <div className="flex flex-col">
              {/* {orderbook.bidDepth.map(([price, total]) => ( */}
                <OrderbookEntry
                  key={`bid-${price}`}
                  price={price.toFixed(2)}
                  total={total}
                  type="bid"
                />
              {/* ))} */}
            </div>
          </div>
        </div>
      </div>
    );
  };
  
  interface OrderbookEntryProps {
    price: string;
    total: number;
    type: "ask" | "bid";
  }
  
  const OrderbookEntry = ({ price, total, type }: OrderbookEntryProps) => {
    const textColor = type === "ask" ? "text-red-400/90" : "text-green-400/90";
  
    return (
      <div className="flex h-[25px] items-center">
        <button type="button" className="h-full w-full">
          <div className="flex items-center justify-between px-3 h-full w-full overflow-hidden hover:border-dashed hover:border-baseBorderFocus/50">
            <p
              className={`z-10 text-left text-xs font-normal tabular-nums ${textColor}`}
            >
              {price}
            </p>
            <p className="z-10 text-right text-xs font-normal tabular-nums text-baseTextHighEmphasis/80">
              {total}
            </p>
          </div>
        </button>
      </div>
    );
  };
  
  interface OrderbookMidpointProps {
    price: number;
  }
  
  const OrderbookMidpoint = ({ price }: OrderbookMidpointProps) => {
    return (
      <div className="flex flex-col snap-center px-3 py-1">
        <div className="flex justify-between">
          <p className="font-medium tabular-nums text-redText">{price}</p>
        </div>
      </div>
    );
  };