import { Order, Quantity } from "./types";

class LinkedListNode<T extends Order> {
  val: T;
  next: LinkedListNode<T> | null;
  prev: LinkedListNode<T> | null;

  constructor(val: T) {
    this.val = val;
    this.next = null;
    this.prev = null;
  }
}

interface List<T extends Order> {
  head: LinkedListNode<T>;
  tail: LinkedListNode<T>;
  size: number;
}

class OrderLinkedList<T extends Order> implements Iterable<T> {
  private list: List<T> | undefined;

  constructor() {
    this.list = undefined;
  }

  size(): number {
    if (this.list) return this.list.size;

    return 0;
  }

  isEmpty(): boolean {
    return !this.list;
  }

  addFront(val: T): boolean {
    const newNode = new LinkedListNode(val);

    if (this.list) {
      // link old head backwards
      this.list.head.prev = newNode;

      // link new head forwards
      newNode.next = this.list.head;

      this.list.head = newNode;
      this.list.size += 1;
    } else {
      this.list = {
        head: newNode,
        tail: newNode,
        size: 1,
      };
    }

    return true;
  }

  addBack(val: T): boolean {
    const newNode = new LinkedListNode(val);

    if (this.list) {
      // link old tail forwards
      this.list.tail.next = newNode;

      // link new tail backwards
      newNode.prev = this.list.tail;

      this.list.tail = newNode;
      this.list.size += 1;
    } else {
      this.list = {
        head: newNode,
        tail: newNode,
        size: 1,
      };
    }

    return true;
  }

  addSellOrders(val: T): boolean {
    const newNode = new LinkedListNode(val);
    if (!this.list) {
      this.list = { head: newNode, tail: newNode, size: 1 };
      return true;
    }
    if (val.orderPrice < this.list.head.val.orderPrice) {
      this.addFront(val);
      return true;
    }
    if (val.orderPrice >= this.list.tail.val.orderPrice) {
      this.addBack(val);
      return true;
    }

    // find the correct position to insert
    let cur = this.list.head;
    while (cur.next && cur.next.val.orderPrice < val.orderPrice) {
      cur = cur.next;
    }
    newNode.next = cur.next;
    if (cur.next) {
      cur.next.prev = newNode;
    }
    cur.next = newNode;
    newNode.prev = cur;
    this.list.size += 1;

    return true;
  }
  addBuyOrders(val: T): boolean {
    const newNode = new LinkedListNode(val);
    if (!this.list) {
      this.list = { head: newNode, tail: newNode, size: 1 };
      return true;
    }

    if (val.orderPrice > this.list.head.val.orderPrice) {
      this.addFront(val);
      return true;
    }

    if (val.orderPrice <= this.list.tail.val.orderPrice) {
      this.addBack(val);
      return true;
    }

    let cur = this.list.head;
    while (cur.next && cur.next.val.orderPrice > val.orderPrice) {
      cur = cur.next;
    }

    newNode.next = cur.next;
    if (cur.next) {
      cur.next.prev = newNode;
    }
    cur.next = newNode;
    newNode.prev = cur;
    this.list.size += 1;

    return true;
  }

  firstItem(): T | null {
    if (!this.list) return null;
    return this.list.head.val;
  }

  lastItem(): T | null {
    if (!this.list) return null;
    return this.list.tail.val;
  }

  removeFront(): T | null {
    if (!this.list) return null;

    // extract val of head so we can return it later
    const val = this.list.head.val;

    if (this.list.head.next) {
      // newHead.prev = null
      this.list.head.next.prev = null;

      // move head pointer forwards
      this.list.head = this.list.head.next;

      this.list.size -= 1;
    } else {
      // list is size 1, clear the list
      this.list = undefined;
    }

    return val;
  }

  removeBack(): T | null {
    if (!this.list) return null;

    // extract the val of tail so we can return it later
    const val = this.list.tail.val;

    if (this.list.tail.prev) {
      // newTail.next = null
      this.list.tail.prev.next = null;

      // move tail pointer backwards
      this.list.tail = this.list.tail.prev;

      this.list.size -= 1;
    } else {
      this.list = undefined;
    }

    return val;
  }

  clear(): void {
    this.list = undefined;
  }

  fromArray(): Array<Order> {
    let array: Order[] = [];
    for (const a of this) {
      array.push(a);
    }
    return array;
  }
  *[Symbol.iterator](): Iterator<T> {
    if (!this.list) return;

    let cur: LinkedListNode<T> | null;

    for (cur = this.list.head; cur != null; cur = cur.next) {
      yield cur.val;
    }
  }
}

export default OrderLinkedList;
