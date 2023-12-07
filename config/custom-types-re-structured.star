my_struct = EchoStruct(
    name = "message",
    fields = {
        "result" : HashMap(String, Int) 
    }
)

attributes = {
    "api_host": "http://127.0.0.1:8080",
    "auth_key": "23bc46b1-71f6-4ed5-8c54-816aa4f8c502:123zO3xZCLrMN6v2BKK1dXYFpXlPkccOFqm12CdAsMgRU4VrNZ9lyGVCGuMDGIwP",
    "insecure": "true",
    "namespace": "guest",
}

cartype = task(
    kind = "openwhisk",
    action_name = "cartype",
    input_arguments = [
        arg(
            name="car_type",
            input_type = String
        ),
    ],
    attributes = attributes
)

modelavail = task(
    kind = "openwhisk",
    action_name = "modelavail",
    input_arguments = [
        arg(
            name="car_company_list",
            input_type = HashMap(String, List(String))
        ),
        arg(
            name="company_name",
            input_type= String
        )
    ],
    attributes = attributes,
    depend_on = [
        depend(task_name = "cartype", cur_field = "car_company_list", prev_field = "car_company_list")
    ]
)

modelprice = task(
    kind = "openwhisk",
    action_name = "modelsprice",
    input_arguments = [
        arg(
            name="models",
            input_type= List(String)
        ),
    ],
    attributes = attributes,
    depend_on = [
        depend(task_name = "modelavail", cur_field = "models", prev_field = "models")
    ]
)

purchase = task(
    kind = "openwhisk",
    action_name = "purchase",
    input_arguments = [
        arg(
            name="model_price_list",
            input_type = my_struct
        ),
        arg(
            name="model_name",
            input_type= String
        ),
        arg(
            name="price",
            input_type= Int
        ),
    ],
    attributes = attributes,
    depend_on = [
        depend(task_name = "modelsprice", cur_field = "model_price_list", prev_field = "model_price_list")
    ]
)

workflows(
    name = "car_market_place",
    version = "0.0.1",
    tasks = [cartype, modelavail, modelprice, purchase]
)