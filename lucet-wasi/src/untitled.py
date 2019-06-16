class Node
    def __init__(self, name):
        self.name = name
        self.conversions = [] # [(name, conversion)]


units = {}


data = [
["Km" 0.621371, "Mile"],
["Yard" 0.9144, "Meter"],
["YEN" 0.0091, "USD"],
["Liter" 0.264172, "Gallon"],
["EURO" 123.27, "YEN"],
["BRL" 0.23, "EURO"],]


def init(data):
    for d in data:
        frm, conversion, too = d
        node = units.get(d[0])
        if not node:
            node = Node(d[0])




