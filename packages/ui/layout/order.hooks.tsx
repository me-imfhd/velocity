"use client";
import {
  createContext,
  useState,
  useContext,
  Dispatch,
  SetStateAction,
} from "react";

interface OrderContextType {
  isBid: boolean;
  setIsBid: Dispatch<SetStateAction<boolean>>;
  isLimit: boolean;
  setIsLimit: Dispatch<SetStateAction<boolean>>;
  price: number;
  setPrice: Dispatch<SetStateAction<number>>;
  quantity: number;
  setQuantity: Dispatch<SetStateAction<number>>;
  marketOrderQuantity: number | number;
  setMarketOrderQuantity: Dispatch<SetStateAction<number | number>>;
}

export const OrderContext = createContext<OrderContextType>(
  {} as OrderContextType
);

interface OrderProviderProps {
  children: React.ReactNode;
}
export const OrderProvider = ({ children }: OrderProviderProps) => {
  const ltsPrice = 99;
  const [isBid, setIsBid] = useState(true);
  const [isLimit, setIsLimit] = useState(true);
  const [price, setPrice] = useState(ltsPrice);
  const [quantity, setQuantity] = useState(0);
  const [marketOrderQuantity, setMarketOrderQuantity] = useState(0);
  return (
    <OrderContext.Provider
      value={{
        isBid,
        setIsBid,
        isLimit,
        setIsLimit,
        price,
        setPrice,
        quantity,
        setQuantity,
        marketOrderQuantity,
        setMarketOrderQuantity,
      }}
    >
      {children}
    </OrderContext.Provider>
  );
};

export const useOrderContext = () => {
  const context = useContext(OrderContext);
  if (context === undefined) {
    throw new Error("use useOrderContext hook within OrderContextProvider");
  }
  return context;
};
